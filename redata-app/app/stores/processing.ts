import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ProcessingTask, ProcessingProgress } from '~/types'
import { processingApi } from '~/utils/api'

export const useProcessingStore = defineStore('processing', () => {
  // State
  const tasks = ref<ProcessingTask[]>([])
  const activeTask = ref<ProcessingTask | null>(null)
  const progress = ref<ProcessingProgress | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const activeTasks = computed(() =>
    tasks.value.filter(t => t.status === 'processing' || t.status === 'paused')
  )
  const hasActiveTasks = computed(() => activeTasks.value.length > 0)
  const completedTasks = computed(() =>
    tasks.value.filter(t => t.status === 'completed')
  )

  // Actions
  async function fetchTasks(projectId: number) {
    loading.value = true
    error.value = null
    try {
      tasks.value = await processingApi.list(projectId)
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
      tasks.value.unshift(task)
      activeTask.value = task
      return task
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
      updateTaskInList(task)
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
      updateTaskInList(task)
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
      updateTaskInList(task)
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
      tasks.value[taskIndex] = {
        ...tasks.value[taskIndex],
        progress: progressData.progress,
        processed_rows: progressData.processed_rows,
        total_rows: progressData.total_rows,
        status: progressData.status,
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
    // Getters
    activeTasks,
    hasActiveTasks,
    completedTasks,
    // Actions
    fetchTasks,
    startProcessing,
    pauseTask,
    resumeTask,
    cancelTask,
    updateProgress,
    clearTasks,
    clearError,
  }
})
