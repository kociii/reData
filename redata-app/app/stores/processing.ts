import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
  TaskProgress, FileProgress, SheetProgress,
  ProcessingProgress, TaskPhase, FilePhase, SheetPhase,
  FullTaskProgressResponse,
} from '~/types'
import { processingApi } from '~/utils/api'

export const useProcessingStore = defineStore('processing', () => {
  // ── State ─────────────────────────────────────────────────────────────────────
  // 使用 Map 存储任务，taskIds 保持有序（新任务在前）
  const taskMap = ref<Map<string, TaskProgress>>(new Map())
  const taskIds = ref<string[]>([])
  const selectedTaskId = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const currentProjectId = ref<number | null>(null)

  // 追踪每个任务当前正在处理的文件/Sheet（row_processed 事件不携带此信息）
  const activeLocation = ref<Map<string, { file: string; sheet: string }>>(new Map())

  // 追踪每个文件开始处理时的累计基线（用于从 file_complete 的累计值计算单文件统计）
  // key: `${taskId}:${fileName}`
  const fileBaseline = ref<Map<string, { success: number; error: number; processed: number }>>(new Map())

  // Tauri 事件监听器清理函数
  let unlistenProgress: (() => void) | null = null

  // ── Map 响应式辅助函数 ────────────────────────────────────────────────────────
  // 每次更新都创建新 Map 对象，确保 Vue 追踪到变化

  function setTask(task: TaskProgress) {
    const newMap = new Map(taskMap.value)
    newMap.set(task.taskId, task)
    taskMap.value = newMap
  }

  function updateTask(taskId: string, updater: (task: TaskProgress) => TaskProgress) {
    const task = taskMap.value.get(taskId)
    if (!task) return
    setTask(updater({ ...task }))
  }

  function updateFile(taskId: string, fileName: string, updater: (file: FileProgress) => FileProgress) {
    const task = taskMap.value.get(taskId)
    if (!task) return
    const idx = task.files.findIndex(f => f.fileName === fileName)
    if (idx === -1) return
    const newFiles = [...task.files]
    const existingFile = newFiles[idx]
    if (!existingFile) return
    newFiles[idx] = updater({ ...existingFile })
    setTask({ ...task, files: newFiles })
  }

  function updateSheet(
    taskId: string,
    fileName: string,
    sheetName: string,
    updater: (sheet: SheetProgress) => SheetProgress,
  ) {
    const task = taskMap.value.get(taskId)
    if (!task) return
    const fileIdx = task.files.findIndex(f => f.fileName === fileName)
    if (fileIdx === -1) return
    const file = task.files[fileIdx]
    if (!file) return
    const sheetIdx = file.sheets.findIndex(s => s.sheetName === sheetName)
    if (sheetIdx === -1) return
    const newSheets = [...file.sheets]
    const existingSheet = newSheets[sheetIdx]
    if (!existingSheet) return
    newSheets[sheetIdx] = updater({ ...existingSheet })
    const newFiles = [...task.files]
    newFiles[fileIdx] = { ...file, sheets: newSheets }
    setTask({ ...task, files: newFiles })
  }

  function setActiveLocation(taskId: string, file: string, sheet: string) {
    const newMap = new Map(activeLocation.value)
    newMap.set(taskId, { file, sheet })
    activeLocation.value = newMap
  }

  // ── 状态映射辅助 ──────────────────────────────────────────────────────────────

  function mapStatusToPhase(status: string): TaskPhase {
    const map: Record<string, TaskPhase> = {
      pending: 'starting',
      processing: 'processing',
      paused: 'paused',
      completed: 'completed',
      cancelled: 'cancelled',
      error: 'error',
      interrupted: 'interrupted',
    }
    return map[status] ?? 'processing'
  }

  function mapStatusToFilePhase(status: string): FilePhase {
    if (status === 'completed') return 'done'
    if (status === 'error') return 'error'
    return 'waiting'
  }

  // ── Getters ───────────────────────────────────────────────────────────────────

  const tasks = computed(() =>
    taskIds.value.map(id => taskMap.value.get(id)!).filter(Boolean),
  )
  const activeTasks = computed(() =>
    tasks.value.filter(t => t.phase === 'processing' || t.phase === 'paused'),
  )
  const completedTasks = computed(() =>
    tasks.value.filter(t => ['completed', 'cancelled', 'error', 'interrupted'].includes(t.phase)),
  )
  const hasActiveTasks = computed(() => activeTasks.value.length > 0)
  const selectedTask = computed(() =>
    selectedTaskId.value ? (taskMap.value.get(selectedTaskId.value) ?? null) : null,
  )

  // ── Actions ───────────────────────────────────────────────────────────────────

  function selectTask(taskId: string | null) {
    selectedTaskId.value = taskId
  }

  // 从后端响应构建完整的 FileProgress 数组
  function buildFilesFromProgressResponse(response: FullTaskProgressResponse): FileProgress[] {
    return response.files.map(file => ({
      fileName: file.file_name,
      phase: file.file_phase as FilePhase,
      sheets: file.sheets.map(sheet => ({
        sheetName: sheet.sheet_name,
        phase: sheet.sheet_phase as SheetPhase,
        aiConfidence: sheet.ai_confidence,
        mappingCount: sheet.mapping_count,
        errorMessage: sheet.error_message,
        successCount: sheet.success_count,
        errorCount: sheet.error_count,
        totalRows: sheet.total_rows,
      })),
      totalRows: file.total_rows,
      successCount: file.success_count,
      errorCount: file.error_count,
    }))
  }

  // 获取任务完整进度（从数据库恢复）
  async function fetchTaskFullProgress(taskId: string): Promise<FileProgress[] | null> {
    try {
      const response = await processingApi.getFullProgress(taskId)
      return buildFilesFromProgressResponse(response)
    }
    catch (e) {
      console.error('[Processing] Failed to fetch full progress:', e)
      return null
    }
  }

  async function fetchTasks(projectId: number) {
    // 切换项目时清理旧状态
    if (currentProjectId.value !== null && currentProjectId.value !== projectId) {
      stopEventListener()
      taskMap.value = new Map()
      taskIds.value = []
      selectedTaskId.value = null
    }
    currentProjectId.value = projectId
    loading.value = true
    error.value = null
    try {
      const response = await processingApi.list(projectId)
      const newMap = new Map<string, TaskProgress>()
      const ids: string[] = []

      for (const task of response.tasks) {
        // 基础任务进度
        let taskProgress: TaskProgress = {
          taskId: task.task_id,
          projectId: task.project_id,
          batchNumber: task.batch_number,
          phase: mapStatusToPhase(task.status),
          sourceFiles: task.source_files || [],
          files: (task.source_files || []).map(fileName => ({
            fileName,
            phase: mapStatusToFilePhase(task.status),
            sheets: [],
            totalRows: 0,
            successCount: 0,
            errorCount: 0,
          })),
          totalRows: task.total_rows,
          processedRows: task.processed_rows,
          successCount: task.success_count,
          errorCount: task.error_count,
          startedAt: task.started_at || new Date().toISOString(),
          completedAt: task.status === 'completed' ? new Date().toISOString() : null,
        }

        // 对于非 pending 状态的任务，尝试从数据库恢复详细进度
        if (task.status !== 'pending') {
          const fullFiles = await fetchTaskFullProgress(task.task_id)
          if (fullFiles && fullFiles.length > 0) {
            taskProgress = {
              ...taskProgress,
              files: fullFiles,
            }
          }
        }

        newMap.set(task.task_id, taskProgress)
        ids.push(task.task_id)
      }

      taskMap.value = newMap
      taskIds.value = ids
    }
    catch (e: any) {
      error.value = e.message
      console.error('Failed to fetch tasks:', e)
    }
    finally {
      loading.value = false
    }
  }

  async function startProcessing(projectId: number, filePaths: string[]) {
    loading.value = true
    error.value = null
    try {
      const task = await processingApi.start({
        project_id: projectId,
        file_paths: filePaths,
      })

      const sourceFileNames = filePaths.map((p) => {
        const parts = p.split(/[/\\]/)
        return parts[parts.length - 1] || p
      })

      const taskProgress: TaskProgress = {
        taskId: task.task_id,
        projectId: task.project_id,
        batchNumber: task.batch_number,
        phase: 'processing',
        sourceFiles: task.source_files || sourceFileNames,
        files: (task.source_files || sourceFileNames).map(fileName => ({
          fileName,
          phase: 'waiting',
          sheets: [],
          totalRows: 0,
          successCount: 0,
          errorCount: 0,
        })),
        totalRows: 0,
        processedRows: 0,
        successCount: 0,
        errorCount: 0,
        startedAt: new Date().toISOString(),
        completedAt: null,
      }

      setTask(taskProgress)
      taskIds.value = [task.task_id, ...taskIds.value]
      selectedTaskId.value = task.task_id
      return taskProgress
    }
    catch (e: any) {
      error.value = e.message
      throw e
    }
    finally {
      loading.value = false
    }
  }

  async function pauseTask(taskId: string) {
    try {
      await processingApi.pause(taskId)
      updateTask(taskId, task => ({ ...task, phase: 'paused' }))
    }
    catch (e: any) {
      error.value = e.message
      throw e
    }
  }

  async function resumeTask(taskId: string) {
    try {
      await processingApi.resume(taskId)
      updateTask(taskId, task => ({ ...task, phase: 'processing' }))
    }
    catch (e: any) {
      error.value = e.message
      throw e
    }
  }

  async function cancelTask(taskId: string) {
    try {
      await processingApi.cancel(taskId)
      updateTask(taskId, task => ({ ...task, phase: 'cancelled' }))
    }
    catch (e: any) {
      error.value = e.message
      throw e
    }
  }

  async function resetTask(taskId: string, deleteRecords: boolean) {
    try {
      const result = await processingApi.reset(taskId, deleteRecords)
      // 重置任务状态到初始状态
      updateTask(taskId, task => ({
        ...task,
        phase: 'starting',
        files: task.files.map(f => ({
          ...f,
          phase: 'waiting' as FilePhase,
          sheets: [],
          totalRows: 0,
          successCount: 0,
          errorCount: 0,
        })),
        totalRows: 0,
        processedRows: 0,
        successCount: 0,
        errorCount: 0,
      }))
      return result
    }
    catch (e: any) {
      error.value = e.message
      throw e
    }
  }

  // ── 进度事件处理 ──────────────────────────────────────────────────────────────

  function handleProgressEvent(data: ProcessingProgress) {
    console.log('[Processing] Event:', data.event, data.task_id)
    const taskId = data.task_id
    if (!taskId) return

    switch (data.event) {
      case 'file_start': {
        const fileName = data.current_file!
        const task = taskMap.value.get(taskId)
        if (!task) break

        // 记录此文件开始时的累计基线，用于后续计算单文件统计
        const baselineMap = new Map(fileBaseline.value)
        baselineMap.set(`${taskId}:${fileName}`, {
          success: task.successCount,
          error: task.errorCount,
          processed: task.processedRows,
        })
        fileBaseline.value = baselineMap

        const existingIdx = task.files.findIndex(f => f.fileName === fileName)
        if (existingIdx !== -1) {
          updateFile(taskId, fileName, f => ({ ...f, phase: 'processing' }))
        }
        else {
          updateTask(taskId, t => ({
            ...t,
            files: [...t.files, {
              fileName,
              phase: 'processing',
              sheets: [],
              totalRows: 0,
              successCount: 0,
              errorCount: 0,
            }],
          }))
        }
        break
      }

      case 'sheet_start': {
        const fileName = data.current_file!
        const sheetName = data.current_sheet!
        setActiveLocation(taskId, fileName, sheetName)

        const task = taskMap.value.get(taskId)
        if (!task) break
        const fileIdx = task.files.findIndex(f => f.fileName === fileName)
        if (fileIdx === -1) break
        const file = task.files[fileIdx]
        if (!file) break
        const existingSheetIdx = file.sheets.findIndex(s => s.sheetName === sheetName)

        if (existingSheetIdx !== -1) {
          updateSheet(taskId, fileName, sheetName, s => ({ ...s, phase: 'ai_analyzing' }))
        }
        else {
          updateFile(taskId, fileName, f => ({
            ...f,
            sheets: [...f.sheets, {
              sheetName,
              phase: 'ai_analyzing',
              aiConfidence: null,
              mappingCount: null,
              errorMessage: null,
              successCount: 0,
              errorCount: 0,
              totalRows: 0,
            }],
          }))
        }
        break
      }

      case 'ai_analyzing':
        // sheet_start 已设置 ai_analyzing 阶段，此处无需额外操作
        break

      case 'ai_request':
      case 'ai_response':
        // 完全丢弃，不做任何 UI 处理
        break

      case 'column_mapping': {
        const sheetName = data.current_sheet!
        const loc = activeLocation.value.get(taskId)
        if (!loc) break
        updateSheet(taskId, loc.file, sheetName, s => ({
          ...s,
          phase: 'importing',
          aiConfidence: data.confidence ?? null,
          mappingCount: data.mappings ? Object.keys(data.mappings).length : null,
        }))
        break
      }

      case 'row_processed': {
        // row_processed 携带的是当前文件内的局部计数（非累计），需加上基线得到任务级累计
        const localProcessed = data.processed_rows ?? 0
        const localSuccess = data.success_count ?? 0
        const localError = data.error_count ?? 0
        const loc = activeLocation.value.get(taskId)
        const baseline = loc ? fileBaseline.value.get(`${taskId}:${loc.file}`) : null
        updateTask(taskId, t => ({
          ...t,
          processedRows: (baseline?.processed ?? 0) + localProcessed,
          successCount: (baseline?.success ?? 0) + localSuccess,
          errorCount: (baseline?.error ?? 0) + localError,
        }))
        break
      }

      case 'sheet_complete': {
        const sheetName = data.current_sheet!
        const loc = activeLocation.value.get(taskId)
        if (!loc) break
        updateSheet(taskId, loc.file, sheetName, s => ({
          ...s,
          phase: 'done',
          successCount: data.sheet_success_count ?? s.successCount,
          errorCount: data.sheet_error_count ?? s.errorCount,
          totalRows: data.sheet_total_rows ?? s.totalRows,
        }))
        break
      }

      case 'file_complete': {
        const fileName = data.current_file!
        // file_complete 发送的是累计值（跨所有已处理文件），需减去此文件的基线得到单文件统计
        const cumulativeSuccess = data.success_count ?? 0
        const cumulativeError = data.error_count ?? 0
        const cumulativeProcessed = data.processed_rows ?? 0
        const baseline = fileBaseline.value.get(`${taskId}:${fileName}`)
        const fileSuccess = cumulativeSuccess - (baseline?.success ?? 0)
        const fileError = cumulativeError - (baseline?.error ?? 0)
        updateFile(taskId, fileName, f => ({
          ...f,
          phase: 'done',
          totalRows: fileSuccess + fileError,
          successCount: fileSuccess,
          errorCount: fileError,
        }))
        updateTask(taskId, t => ({
          ...t,
          processedRows: cumulativeProcessed,
          successCount: cumulativeSuccess,
          errorCount: cumulativeError,
        }))
        break
      }

      case 'completed': {
        const successCount = data.success_count ?? 0
        const errorCount = data.error_count ?? 0
        updateTask(taskId, t => ({
          ...t,
          phase: 'completed',
          processedRows: data.processed_rows ?? t.processedRows,
          successCount,
          errorCount,
          completedAt: new Date().toISOString(),
          files: t.files.map(f => ({
            ...f,
            phase: 'done' as FilePhase,
            sheets: f.sheets.map(s => ({
              ...s,
              phase: 'done' as SheetPhase,
              // 保留已有的统计数据
              successCount: s.successCount,
              errorCount: s.errorCount,
              totalRows: s.totalRows,
            })),
          })),
        }))
        break
      }

      case 'error': {
        const fileName = data.current_file
        const loc = activeLocation.value.get(taskId)
        if (fileName) {
          // 文件级错误
          updateFile(taskId, fileName, f => ({ ...f, phase: 'error' }))
        }
        else if (loc) {
          // Sheet 级错误
          updateSheet(taskId, loc.file, loc.sheet, s => ({
            ...s,
            phase: 'error',
            errorMessage: data.message ?? null,
          }))
        }
        else {
          // 任务级错误
          updateTask(taskId, t => ({ ...t, phase: 'error' }))
        }
        break
      }

      default:
        break
    }
  }

  // ── Tauri 事件监听 ────────────────────────────────────────────────────────────

  async function startEventListener() {
    if (unlistenProgress) return
    try {
      unlistenProgress = await processingApi.onProgress(handleProgressEvent)
      console.log('[Processing] Event listener started')
    }
    catch (e) {
      console.error('[Processing] Failed to start event listener:', e)
    }
  }

  function stopEventListener() {
    if (unlistenProgress) {
      unlistenProgress()
      unlistenProgress = null
      console.log('[Processing] Event listener stopped')
    }
  }

  // ── 状态轮询（兜底机制，防止事件丢失）────────────────────────────────────────

  let pollTimer: ReturnType<typeof setInterval> | null = null

  async function syncTaskStatus(taskId: string) {
    try {
      const task = await processingApi.status(taskId)
      updateTask(taskId, existing => ({
        ...existing,
        phase: mapStatusToPhase(task.status),
        totalRows: task.total_rows,
        processedRows: task.processed_rows,
        successCount: task.success_count,
        errorCount: task.error_count,
      }))
    }
    catch {
      // 静默失败，不影响主流程
    }
  }

  function startStatusPolling() {
    stopStatusPolling()
    pollTimer = setInterval(() => {
      activeTasks.value.forEach(task => syncTaskStatus(task.taskId))
    }, 10000)
  }

  function stopStatusPolling() {
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    taskMap,
    taskIds,
    selectedTaskId,
    loading,
    error,
    // Getters
    tasks,
    activeTasks,
    completedTasks,
    hasActiveTasks,
    selectedTask,
    // Actions
    selectTask,
    fetchTasks,
    startProcessing,
    pauseTask,
    resumeTask,
    cancelTask,
    resetTask,
    clearError,
    // 事件监听
    startEventListener,
    stopEventListener,
    handleProgressEvent,
    // 状态轮询
    startStatusPolling,
    stopStatusPolling,
  }
})
