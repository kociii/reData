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

  // 新增：WebSocket 连接管理
  const wsConnections = ref<Map<string, WebSocket>>(new Map())

  // WebSocket 重连控制
  const wsRetryCount = ref<Map<string, number>>(new Map())
  const wsRetryTimers = ref<Map<string, ReturnType<typeof setTimeout>>>(new Map())
  const WS_MAX_RETRIES = 5
  const WS_RETRY_DELAY = 3000 // 3 秒

  // 新增：选中的任务 ID（右侧面板展示）
  const selectedTaskId = ref<string | null>(null)

  // 新增：每个任务的处理阶段 Map<taskId, ProcessingStage[]>
  const taskStages = ref<Map<string, ProcessingStage[]>>(new Map())

  // Getters
  const activeTasks = computed(() =>
    tasks.value.filter(t => t.status === 'processing' || t.status === 'paused')
  )
  const hasActiveTasks = computed(() => activeTasks.value.length > 0)
  const completedTasks = computed(() =>
    tasks.value.filter(t => t.status === 'completed')
  )
  const hasPendingFiles = computed(() => pendingFiles.value.length > 0)
  const selectedTask = computed(() =>
    selectedTaskId.value ? tasks.value.find(t => t.id === selectedTaskId.value) : null
  )
  const selectedStages = computed(() =>
    selectedTaskId.value ? taskStages.value.get(selectedTaskId.value) || createDefaultStages() : createDefaultStages()
  )

  // Actions
  async function fetchTasks(projectId: number) {
    // 切换项目时清理旧状态
    if (currentProjectId.value !== null && currentProjectId.value !== projectId) {
      disconnectAllWebSockets()
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
      // 添加 id 字段
      const taskWithId = { ...task, id: task.task_id }
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
      const task = await processingApi.pause(taskId)
      const taskWithId = { ...task, id: task.task_id }
      updateTaskInList(taskWithId)
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
      const task = await processingApi.resume(taskId)
      const taskWithId = { ...task, id: task.task_id }
      updateTaskInList(taskWithId)
      // 恢复时重新连接 WebSocket
      connectWebSocket(taskId)
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
      const task = await processingApi.cancel(taskId)
      const taskWithId = { ...task, id: task.task_id }
      updateTaskInList(taskWithId)
      // 取消时断开 WebSocket
      disconnectWebSocket(taskId)
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
  function addLog(log: LogEntry) {
    logs.value.push(log)
    // 保持最近 200 条日志
    if (logs.value.length > 200) {
      logs.value.shift()
    }
  }

  function clearLogs() {
    logs.value = []
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
  }

  function updateTaskStage(taskId: string, stageKey: string, status: ProcessingStage['status']) {
    const stages = taskStages.value.get(taskId)
    if (!stages) return
    const stage = stages.find(s => s.key === stageKey)
    if (stage) stage.status = status
  }

  function selectTask(taskId: string | null) {
    selectedTaskId.value = taskId
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
          disconnectWebSocket(taskId)
          addLog({
            time: new Date().toLocaleTimeString('zh-CN'),
            message: `任务状态已同步: ${task.status === 'completed' ? '已完成' : task.status}`,
            type: task.status === 'completed' ? 'success' : 'warning',
          })
          // 更新阶段
          if (task.status === 'completed') {
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

  // WebSocket 连接管理
  function connectWebSocket(taskId: string) {
    // 如果已存在连接，不重复连接
    if (wsConnections.value.has(taskId)) {
      return
    }

    // 检查重试次数
    const retries = wsRetryCount.value.get(taskId) || 0
    if (retries >= WS_MAX_RETRIES) {
      addLog({
        time: new Date().toLocaleTimeString('zh-CN'),
        message: `任务 ${taskId.slice(0, 8)}... WebSocket 重连次数已达上限`,
        type: 'warning',
      })
      return
    }

    try {
      const ws = processingApi.connectProgress(taskId)

      ws.onopen = () => {
        // 连接成功，重置重试计数
        wsRetryCount.value.set(taskId, 0)
        addLog({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: `已连接到任务 ${taskId.slice(0, 8)}... 的进度流`,
          type: 'info',
        })
        // 重连后立即从 API 同步任务状态（防止错过 completed 事件）
        syncTaskStatus(taskId)
      }

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data)
          handleWebSocketMessage(data)
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e)
        }
      }

      ws.onerror = () => {
        // 只记录日志，不在这里重连（由 onclose 处理）
      }

      ws.onclose = () => {
        wsConnections.value.delete(taskId)

        // 检查任务是否仍在活动状态，如果是则延迟重连
        const task = tasks.value.find(t => t.id === taskId)
        if (task && (task.status === 'processing' || task.status === 'paused')) {
          const currentRetries = (wsRetryCount.value.get(taskId) || 0) + 1
          wsRetryCount.value.set(taskId, currentRetries)

          if (currentRetries < WS_MAX_RETRIES) {
            addLog({
              time: new Date().toLocaleTimeString('zh-CN'),
              message: `进度流断开，${WS_RETRY_DELAY / 1000}s 后重连 (${currentRetries}/${WS_MAX_RETRIES})`,
              type: 'warning',
            })
            const timer = setTimeout(() => {
              wsRetryTimers.value.delete(taskId)
              connectWebSocket(taskId)
            }, WS_RETRY_DELAY)
            wsRetryTimers.value.set(taskId, timer)
          }
        }
      }

      wsConnections.value.set(taskId, ws)
    } catch (e) {
      console.error('Failed to connect WebSocket:', e)
    }
  }

  function disconnectWebSocket(taskId: string) {
    // 清除重连定时器
    const timer = wsRetryTimers.value.get(taskId)
    if (timer) {
      clearTimeout(timer)
      wsRetryTimers.value.delete(taskId)
    }
    wsRetryCount.value.delete(taskId)

    const ws = wsConnections.value.get(taskId)
    if (ws) {
      ws.onclose = null // 防止触发重连逻辑
      ws.close()
      wsConnections.value.delete(taskId)
    }
  }

  function disconnectAllWebSockets() {
    // 清除所有重连定时器
    wsRetryTimers.value.forEach(timer => clearTimeout(timer))
    wsRetryTimers.value.clear()
    wsRetryCount.value.clear()

    wsConnections.value.forEach((ws) => {
      ws.onclose = null
      ws.close()
    })
    wsConnections.value.clear()
  }

  function handleWebSocketMessage(data: any) {
    const taskId = data.task_id
    // 确保阶段已初始化
    if (taskId && !taskStages.value.has(taskId)) {
      initTaskStages(taskId)
    }

    switch (data.event) {
      case 'file_start':
      case 'sheet_start':
        if (taskId) {
          updateTaskStage(taskId, 'preparing', 'completed')
          updateTaskStage(taskId, 'ai_mapping', 'active')
        }
        addLog({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: data.message || `开始处理: ${data.current_sheet || data.current_file || ''}`,
          type: 'info',
        })
        break

      case 'column_mapping':
        if (taskId) {
          updateTaskStage(taskId, 'ai_mapping', 'completed')
          updateTaskStage(taskId, 'importing', 'active')
        }
        addLog({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: `AI 列映射完成 (置信度: ${((data.confidence || 0) * 100).toFixed(0)}%)`,
          type: 'success',
        })
        break

      case 'ai_analyzing':
        addLog({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: data.message || 'AI 分析中...',
          type: 'info',
        })
        break

      case 'row_processed':
        updateProgress(data)
        // 每 50 行记录一次日志，避免刷屏
        if (data.processed_rows && data.processed_rows % 50 === 0) {
          addLog({
            time: new Date().toLocaleTimeString('zh-CN'),
            message: `导入进度: ${data.processed_rows}/${data.total_rows}`,
            type: 'info',
          })
        }
        break

      case 'sheet_complete':
      case 'file_complete':
        addLog({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: data.message || `处理完成: ${data.current_sheet || data.current_file || ''}`,
          type: 'success',
        })
        break

      case 'completed':
        if (taskId) {
          updateTaskStage(taskId, 'importing', 'completed')
          updateTaskStage(taskId, 'done', 'completed')
        }
        updateProgress(data)
        addLog({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: `任务完成: 成功 ${data.success_count}, 失败 ${data.error_count}`,
          type: 'success',
        })
        if (taskId) {
          disconnectWebSocket(taskId)
        }
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
        addLog({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: `错误: ${data.message}`,
          type: 'error',
        })
        break

      case 'warning':
        addLog({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: `警告: ${data.message}`,
          type: 'warning',
        })
        break

      default:
        addLog({
          time: new Date().toLocaleTimeString('zh-CN'),
          message: data.message || JSON.stringify(data),
          type: 'info',
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
    wsConnections,
    selectedTaskId,
    taskStages,
    // Getters
    activeTasks,
    hasActiveTasks,
    completedTasks,
    hasPendingFiles,
    selectedTask,
    selectedStages,
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
    clearLogs,
    // 阶段管理
    initTaskStages,
    updateTaskStage,
    selectTask,
    // 状态同步
    syncTaskStatus,
    startStatusPolling,
    stopStatusPolling,
    // WebSocket
    connectWebSocket,
    disconnectWebSocket,
    disconnectAllWebSockets,
  }
})
