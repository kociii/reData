import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ProcessingTask, ProcessingProgress, PendingFile, LogEntry, ProcessingStage } from '~/types'
import { processingApi } from '~/utils/api'

export const useProcessingStore = defineStore('processing', () => {
  // State
  const tasks = ref<ProcessingTask[]>([])
  const activeTask = ref<ProcessingTask | null>(null)
  const progress = ref<ProcessingProgress | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const currentProjectId = ref<number | null>(null)  // 当前项目 ID

  // 新增：待处理文件列表
  const pendingFiles = ref<PendingFile[]>([])

  // 新增：实时日志记录
  const logs = ref<LogEntry[]>([])

  // Tauri 事件监听器
  let unlistenProgress: (() => void) | null = null

  // 新增：选中的任务 ID（右侧面板展示）
  const selectedTaskId = ref<string | null>(null)

  // 新增：每个任务的处理阶段 Map<taskId, ProcessingStage[]>
  const taskStages = ref<Map<string, ProcessingStage[]>>(new Map())

  // 新增：每个任务的独立日志 Map<taskId, LogEntry[]>（状态隔离）
  const taskLogs = ref<Map<string, LogEntry[]>>(new Map())

  // 新增：多选的任务 ID
  const selectedTaskIds = ref<Set<string>>(new Set())

  // 新增：分组折叠状态
  const collapsedGroups = ref<Set<string>>(new Set())

  // Getters
  // 待处理任务（pending 状态）
  const pendingTasks = computed(() =>
    tasks.value.filter(t => t.status === 'pending')
  )
  // 处理中任务（processing, paused, queued）
  const processingTasks = computed(() =>
    tasks.value.filter(t => t.status === 'processing' || t.status === 'paused' || t.status === 'queued')
  )
  // 已完成任务（completed, cancelled）
  const completedTasks = computed(() =>
    tasks.value.filter(t => t.status === 'completed' || t.status === 'cancelled')
  )
  // 兼容旧代码
  const activeTasks = computed(() => processingTasks.value)
  const hasActiveTasks = computed(() => processingTasks.value.length > 0)
  const hasPendingFiles = computed(() => pendingFiles.value.length > 0)
  const selectedTask = computed(() =>
    selectedTaskId.value ? tasks.value.find(t => t.id === selectedTaskId.value) : null
  )
  const selectedStages = computed(() =>
    selectedTaskId.value ? taskStages.value.get(selectedTaskId.value) || createDefaultStages() : createDefaultStages()
  )
  // 选中任务的独立日志
  const selectedLogs = computed(() =>
    selectedTaskId.value ? taskLogs.value.get(selectedTaskId.value) || [] : []
  )

  // Actions
  async function fetchTasks(projectId: number) {
    // 切换项目时清理旧状态
    if (currentProjectId.value !== null && currentProjectId.value !== projectId) {
      stopEventListener()
      tasks.value = []
      logs.value = []
      pendingFiles.value = []
      selectedTaskId.value = null
      taskStages.value.clear()
      activeTask.value = null
      progress.value = null
    }
    currentProjectId.value = projectId

    loading.value = true
    error.value = null
    try {
      const response = await processingApi.list(projectId)
      tasks.value = response.tasks.map(task => ({
        ...task,
        id: task.task_id,
      }))

      // 为已完成/已取消的任务初始化阶段状态
      tasks.value.forEach(task => {
        if (!taskStages.value.has(task.id)) {
          if (task.status === 'completed' || task.status === 'cancelled') {
            // 所有阶段都标记为完成
            const completedStages = createDefaultStages().map(s => ({
              ...s,
              status: 'completed' as const
            }))
            taskStages.value.set(task.id, completedStages)
          } else if (task.status === 'error') {
            // 错误状态保持默认
            initTaskStages(task.id)
          }
        }
      })
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to fetch tasks:', e)
    } finally {
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
      // 提取源文件名
      const sourceFileNames = filePaths.map(p => {
        const parts = p.split(/[/\\]/)
        return parts[parts.length - 1] || p
      })
      // 添加 id 字段
      const taskWithId: ProcessingTask = {
        id: task.task_id,
        task_id: task.task_id,
        project_id: task.project_id,
        status: task.status,
        total_files: filePaths.length,
        processed_files: 0,
        total_rows: 0,
        processed_rows: 0,
        success_count: 0,
        error_count: 0,
        batch_number: task.batch_number,
        source_files: task.source_files || sourceFileNames,
      }
      tasks.value.unshift(taskWithId)
      activeTask.value = taskWithId
      // 初始化阶段并选中
      initTaskStages(task.task_id)
      updateTaskStage(task.task_id, 'preparing', 'active')
      selectedTaskId.value = task.task_id
      return taskWithId
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to start processing:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function pauseTask(taskId: string) {
    loading.value = true
    error.value = null
    try {
      await processingApi.pause(taskId)
      const taskIndex = tasks.value.findIndex(t => t.id === taskId)
      if (taskIndex !== -1) {
        tasks.value[taskIndex].status = 'paused'
      }
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to pause task:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function resumeTask(taskId: string) {
    loading.value = true
    error.value = null
    try {
      await processingApi.resume(taskId)
      const taskIndex = tasks.value.findIndex(t => t.id === taskId)
      if (taskIndex !== -1) {
        tasks.value[taskIndex].status = 'processing'
      }
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to resume task:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function cancelTask(taskId: string) {
    loading.value = true
    error.value = null
    try {
      await processingApi.cancel(taskId)
      const taskIndex = tasks.value.findIndex(t => t.id === taskId)
      if (taskIndex !== -1) {
        tasks.value[taskIndex].status = 'cancelled'
      }
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to cancel task:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  function updateProgress(progressData: ProcessingProgress) {
    progress.value = progressData

    // 更新任务列表中的对应任务
    const taskIndex = tasks.value.findIndex(t => t.id === progressData.task_id)
    if (taskIndex !== -1) {
      const task = tasks.value[taskIndex]
      const newProgress = progressData.total_rows
        ? Math.round((progressData.processed_rows || 0) / progressData.total_rows * 100)
        : 0

      tasks.value[taskIndex] = {
        ...task,
        progress: newProgress,
        processed_rows: progressData.processed_rows ?? task.processed_rows,
        total_rows: progressData.total_rows ?? task.total_rows,
        success_count: progressData.success_count ?? task.success_count,
        error_count: progressData.error_count ?? task.error_count,
        status: progressData.event === 'completed' ? 'completed'
          : progressData.event === 'error' ? 'error'
          : task.status,
      }
    }
  }

  function updateTaskInList(task: ProcessingTask) {
    const index = tasks.value.findIndex(t => t.id === task.id)
    if (index !== -1) {
      tasks.value[index] = task
    }
    if (activeTask.value?.id === task.id) {
      activeTask.value = task
    }
  }

  // 待处理文件管理
  function addPendingFile(file: PendingFile) {
    // 避免重复添加
    if (!pendingFiles.value.find(f => f.path === file.path)) {
      pendingFiles.value.push(file)
    }
  }

  function addPendingFiles(files: PendingFile[]) {
    files.forEach(file => addPendingFile(file))
  }

  function removePendingFile(fileId: string) {
    const index = pendingFiles.value.findIndex(f => f.id === fileId)
    if (index !== -1) {
      pendingFiles.value.splice(index, 1)
    }
  }

  function clearPendingFiles() {
    pendingFiles.value = []
  }

  // 日志管理
  // 添加日志到全局日志（兼容旧代码）
  function addLog(log: LogEntry) {
    logs.value.push(log)
    // 保持最近 200 条日志
    if (logs.value.length > 200) {
      logs.value.shift()
    }
  }

  // 添加日志到指定任务（状态隔离）
  function addTaskLog(taskId: string, log: LogEntry) {
    // 同时更新全局日志（兼容）
    addLog(log)

    // 更新任务独立日志
    const taskLogList = taskLogs.value.get(taskId) || []
    taskLogList.push(log)
    // 每个任务保持最近 500 条日志
    if (taskLogList.length > 500) {
      taskLogList.shift()
    }
    taskLogs.value.set(taskId, [...taskLogList])
  }

  function clearLogs() {
    logs.value = []
  }

  function clearTaskLogs(taskId: string) {
    taskLogs.value.set(taskId, [])
  }

  // 初始化任务日志
  function initTaskLogs(taskId: string) {
    if (!taskLogs.value.has(taskId)) {
      taskLogs.value.set(taskId, [])
    }
  }

  // 阶段管理
  function createDefaultStages(): ProcessingStage[] {
    return [
      { key: 'preparing', label: '准备中', status: 'pending' },
      { key: 'ai_mapping', label: 'AI 识别', status: 'pending' },
      { key: 'importing', label: '数据导入', status: 'pending' },
      { key: 'done', label: '处理完成', status: 'pending' },
    ]
  }

  function initTaskStages(taskId: string) {
    taskStages.value.set(taskId, createDefaultStages())
    initTaskLogs(taskId)
  }

  function updateTaskStage(taskId: string, stageKey: string, status: ProcessingStage['status']) {
    const stages = taskStages.value.get(taskId)
    if (!stages) return
    const stageIndex = stages.findIndex(s => s.key === stageKey)
    if (stageIndex !== -1) {
      // 创建新数组以确保 Vue 响应式更新
      const newStages = [...stages]
      newStages[stageIndex] = { ...newStages[stageIndex], status }
      taskStages.value.set(taskId, newStages)
    }
  }

  function selectTask(taskId: string | null) {
    selectedTaskId.value = taskId
  }

  // 多选管理
  function toggleTaskSelection(taskId: string) {
    if (selectedTaskIds.value.has(taskId)) {
      selectedTaskIds.value.delete(taskId)
    } else {
      selectedTaskIds.value.add(taskId)
    }
    // 触发响应式更新
    selectedTaskIds.value = new Set(selectedTaskIds.value)
  }

  function selectAllTasks(taskIds: string[]) {
    taskIds.forEach(id => selectedTaskIds.value.add(id))
    selectedTaskIds.value = new Set(selectedTaskIds.value)
  }

  function deselectAllTasks() {
    selectedTaskIds.value = new Set()
  }

  function isSelected(taskId: string): boolean {
    return selectedTaskIds.value.has(taskId)
  }

  // 分组折叠管理
  function toggleGroupCollapse(group: string) {
    if (collapsedGroups.value.has(group)) {
      collapsedGroups.value.delete(group)
    } else {
      collapsedGroups.value.add(group)
    }
    collapsedGroups.value = new Set(collapsedGroups.value)
  }

  function isGroupCollapsed(group: string): boolean {
    return collapsedGroups.value.has(group)
  }

  // 批量操作
  async function batchPauseTasks(taskIds: string[]) {
    for (const taskId of taskIds) {
      try {
        await pauseTask(taskId)
      } catch (e) {
        console.error('Failed to pause task:', taskId, e)
      }
    }
  }

  async function batchResumeTasks(taskIds: string[]) {
    for (const taskId of taskIds) {
      try {
        await resumeTask(taskId)
      } catch (e) {
        console.error('Failed to resume task:', taskId, e)
      }
    }
  }

  async function batchCancelTasks(taskIds: string[]) {
    for (const taskId of taskIds) {
      try {
        await cancelTask(taskId)
      } catch (e) {
        console.error('Failed to cancel task:', taskId, e)
      }
    }
  }

  // 从 API 同步任务状态（兜底机制，防止 WebSocket 丢失事件）
  async function syncTaskStatus(taskId: string) {
    try {
      const task = await processingApi.status(taskId)
      const taskWithId = { ...task, id: task.task_id }
      const index = tasks.value.findIndex(t => t.id === taskId)
      if (index !== -1) {
        const existing = tasks.value[index]
        tasks.value[index] = {
          ...existing,
          ...taskWithId,
          progress: existing.progress, // 保留前端计算的进度
        }
        // 如果后端已完成但前端还在 processing，更新状态并断开 WS
        if (
          (task.status === 'completed' || task.status === 'cancelled' || task.status === 'error') &&
          (existing.status === 'processing' || existing.status === 'paused')
        ) {
          tasks.value[index].status = task.status as any
          addLog({
            time: new Date().toLocaleTimeString('zh-CN'),
            message: `任务状态已同步: ${task.status === 'completed' ? '已完成' : task.status}`,
            type: task.status === 'completed' ? 'success' : 'warning',
          })
          // 更新阶段（确保所有阶段都完成）
          if (task.status === 'completed') {
            updateTaskStage(taskId, 'preparing', 'completed')
            updateTaskStage(taskId, 'ai_mapping', 'completed')
            updateTaskStage(taskId, 'importing', 'completed')
            updateTaskStage(taskId, 'done', 'completed')
          }
        }
      }
    } catch {
      // 静默失败，不影响主流程
    }
  }

  // 定期轮询活动任务状态（每 10 秒）
  let pollTimer: ReturnType<typeof setInterval> | null = null

  function startStatusPolling() {
    stopStatusPolling()
    pollTimer = setInterval(() => {
      const active = tasks.value.filter(t => t.status === 'processing' || t.status === 'paused')
      active.forEach(task => syncTaskStatus(task.id))
    }, 10000)
  }

  function stopStatusPolling() {
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  // Tauri 事件监听
  async function startEventListener() {
    if (unlistenProgress) return // 已监听

    try {
      unlistenProgress = await processingApi.onProgress((data) => {
        handleProgressEvent(data)
      })
      console.log('[Processing] Started event listener')
    } catch (e) {
      console.error('[Processing] Failed to start event listener:', e)
    }
  }

  function stopEventListener() {
    if (unlistenProgress) {
      unlistenProgress()
      unlistenProgress = null
      console.log('[Processing] Stopped event listener')
    }
  }

  function handleProgressEvent(data: ProcessingProgress) {
    console.log('[Processing] Received event:', data.event, data.task_id)
    const taskId = data.task_id
    // 确保阶段已初始化
    if (taskId && !taskStages.value.has(taskId)) {
      initTaskStages(taskId)
    }

    // 辅助函数：添加日志到指定任务
    const addLogForTask = (log: LogEntry) => {
      if (taskId) {
        addTaskLog(taskId, log)
      } else {
        addLog(log)
      }
    }

    switch (data.event) {
      case 'file_start':
      case 'sheet_start':
        if (taskId) {
          updateTaskStage(taskId, 'preparing', 'completed')
          updateTaskStage(taskId, 'ai_mapping', 'active')
        }
        addLogForTask({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: data.message || `开始处理: ${data.current_sheet || data.current_file || ''}`,
          type: 'info',
          align: 'left',
        })
        break

      case 'column_mapping':
        if (taskId) {
          updateTaskStage(taskId, 'ai_mapping', 'completed')
          updateTaskStage(taskId, 'importing', 'active')
        }
        addLogForTask({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: `AI 列映射完成 (置信度: ${((data.confidence || 0) * 100).toFixed(0)}%)`,
          type: 'success',
          align: 'left',
        })
        break

      case 'ai_analyzing':
        addLogForTask({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: data.message || 'AI 分析中...',
          type: 'info',
          align: 'left',
          category: 'ai',
        })
        break

      case 'ai_request':
        // 显示发送给 AI 的请求内容
        if (data.message) {
          addLogForTask({
            time: new Date().toLocaleTimeString('zh-CN'),
            message: data.message,
            type: 'info',
            align: 'left',
            category: 'ai_request',
          })
        }
        break

      case 'ai_response':
        // AI 流式响应（追加到上一条 AI 日志）
        if (taskId && data.message) {
          const taskLogList = taskLogs.value.get(taskId) || []
          // 查找最后一条 AI 类型的日志
          const lastAiLogIndex = taskLogList.findLastIndex(l => l.category === 'ai')
          if (lastAiLogIndex >= 0) {
            // 追加到最后一条日志
            taskLogList[lastAiLogIndex] = {
              ...taskLogList[lastAiLogIndex],
              message: taskLogList[lastAiLogIndex].message + data.message,
            }
            taskLogs.value.set(taskId, [...taskLogList])
          } else {
            // 没有找到 AI 日志，创建新的
            addLogForTask({
              time: new Date().toLocaleTimeString('zh-CN'),
              message: data.message,
              type: 'info',
              align: 'left',
              category: 'ai',
            })
          }
        }
        break

      case 'row_processed':
        updateProgress(data)
        // 每 50 行记录一次日志，避免刷屏
        if (data.processed_rows && data.processed_rows % 50 === 0) {
          addLogForTask({
            time: new Date().toLocaleTimeString('zh-CN'),
            message: `导入进度: ${data.processed_rows}/${data.total_rows}`,
            type: 'info',
            align: 'right',
            category: 'progress',
          })
        }
        break

      case 'sheet_complete':
      case 'file_complete':
        addLogForTask({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: data.message || `处理完成: ${data.current_sheet || data.current_file || ''}`,
          type: 'success',
          align: 'left',
        })
        break

      case 'completed':
        if (taskId) {
          // 任务完成时，确保所有阶段都标记为完成
          updateTaskStage(taskId, 'preparing', 'completed')
          updateTaskStage(taskId, 'ai_mapping', 'completed')
          updateTaskStage(taskId, 'importing', 'completed')
          updateTaskStage(taskId, 'done', 'completed')
        }
        updateProgress(data)
        addLogForTask({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: `任务完成: 成功 ${data.success_count}, 失败 ${data.error_count}`,
          type: 'success',
          align: 'left',
        })
        break

      case 'error':
        if (taskId) {
          // 标记当前活动阶段为错误
          const stages = taskStages.value.get(taskId)
          if (stages) {
            const activeStage = stages.find(s => s.status === 'active')
            if (activeStage) activeStage.status = 'error'
          }
        }
        addLogForTask({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: `错误: ${data.message}`,
          type: 'error',
          align: 'left',
        })
        break

      case 'warning':
        addLogForTask({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: `警告: ${data.message}`,
          type: 'warning',
          align: 'left',
        })
        break

      default:
        addLogForTask({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: data.message || JSON.stringify(data),
          type: 'info',
          align: 'left',
        })
    }
  }

  function clearTasks() {
    tasks.value = []
    activeTask.value = null
    progress.value = null
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    tasks,
    activeTask,
    progress,
    loading,
    error,
    pendingFiles,
    logs,
    selectedTaskId,
    taskStages,
    taskLogs,
    selectedTaskIds,
    collapsedGroups,
    // Getters
    pendingTasks,
    processingTasks,
    activeTasks,
    hasActiveTasks,
    completedTasks,
    hasPendingFiles,
    selectedTask,
    selectedStages,
    selectedLogs,
    // Actions
    fetchTasks,
    startProcessing,
    pauseTask,
    resumeTask,
    cancelTask,
    updateProgress,
    clearTasks,
    clearError,
    // 待处理文件
    addPendingFile,
    addPendingFiles,
    removePendingFile,
    clearPendingFiles,
    // 日志
    addLog,
    addTaskLog,
    clearLogs,
    clearTaskLogs,
    initTaskLogs,
    // 阶段管理
    initTaskStages,
    updateTaskStage,
    selectTask,
    // 多选管理
    toggleTaskSelection,
    selectAllTasks,
    deselectAllTasks,
    isSelected,
    // 分组折叠
    toggleGroupCollapse,
    isGroupCollapsed,
    // 批量操作
    batchPauseTasks,
    batchResumeTasks,
    batchCancelTasks,
    // 状态同步
    syncTaskStatus,
    startStatusPolling,
    stopStatusPolling,
    // Tauri 事件监听
    startEventListener,
    stopEventListener,
    handleProgressEvent,
  }
})
