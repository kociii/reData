import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useProcessingStore } from '~/stores/processing'
import { mockProcessingTask } from '../../fixtures/testData'
import { createMockResponse } from '../../mocks/api'

const API_BASE = 'http://127.0.0.1:8000/api'

describe('processingStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  describe('fetchTasks', () => {
    it('应该成功获取任务列表', async () => {
      const store = useProcessingStore()
      const mockResponse = { tasks: [mockProcessingTask], total: 1 }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(mockResponse))
      global.fetch = mockFetch

      await store.fetchTasks(1)

      expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/processing/list/1?`, expect.any(Object))
      expect(store.tasks.length).toBe(1)
      expect(store.tasks[0].id).toBe(mockProcessingTask.task_id)
      expect(store.loading).toBe(false)
    })
  })

  describe('startProcessing', () => {
    it('应该成功启动处理任务', async () => {
      const store = useProcessingStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(mockProcessingTask))
      global.fetch = mockFetch

      const result = await store.startProcessing(1, ['/path/to/file.xlsx'])

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/processing/start`,
        expect.objectContaining({ method: 'POST' })
      )
      expect(result.task_id).toBe(mockProcessingTask.task_id)
      expect(store.activeTask).not.toBeNull()
      expect(store.tasks.length).toBe(1)
    })

    it('应该处理启动失败的情况', async () => {
      const store = useProcessingStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ detail: '启动失败' }, false, 500))
      global.fetch = mockFetch

      await expect(store.startProcessing(1, ['/path/to/file.xlsx'])).rejects.toThrow()
      expect(store.error).toBeTruthy()
    })
  })

  describe('pauseTask', () => {
    it('应该成功暂停任务', async () => {
      const store = useProcessingStore()
      store.tasks = [{ ...mockProcessingTask, id: mockProcessingTask.task_id }]

      const pausedTask = { ...mockProcessingTask, status: 'paused' as const }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(pausedTask))
      global.fetch = mockFetch

      await store.pauseTask('task-001')

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/processing/pause/task-001`,
        expect.objectContaining({ method: 'POST' })
      )
      expect(store.tasks[0].status).toBe('paused')
    })
  })

  describe('resumeTask', () => {
    it('应该成功恢复任务', async () => {
      const store = useProcessingStore()
      store.tasks = [{ ...mockProcessingTask, id: mockProcessingTask.task_id, status: 'paused' }]

      const resumedTask = { ...mockProcessingTask, status: 'processing' as const }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(resumedTask))
      global.fetch = mockFetch

      await store.resumeTask('task-001')

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/processing/resume/task-001`,
        expect.objectContaining({ method: 'POST' })
      )
      expect(store.tasks[0].status).toBe('processing')
    })
  })

  describe('cancelTask', () => {
    it('应该成功取消任务', async () => {
      const store = useProcessingStore()
      store.tasks = [{ ...mockProcessingTask, id: mockProcessingTask.task_id }]

      const cancelledTask = { ...mockProcessingTask, status: 'cancelled' as const }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(cancelledTask))
      global.fetch = mockFetch

      await store.cancelTask('task-001')

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/processing/cancel/task-001`,
        expect.objectContaining({ method: 'POST' })
      )
      expect(store.tasks[0].status).toBe('cancelled')
    })
  })

  describe('updateProgress', () => {
    it('应该正确更新进度', () => {
      const store = useProcessingStore()
      store.tasks = [{ ...mockProcessingTask, id: mockProcessingTask.task_id }]

      store.updateProgress({
        event: 'row_processed',
        task_id: 'task-001',
        processed_rows: 75,
        total_rows: 100,
        success_count: 70,
        error_count: 5
      })

      expect(store.progress).not.toBeNull()
      expect(store.tasks[0].progress).toBe(75)
      expect(store.tasks[0].processed_rows).toBe(75)
    })

    it('完成任务时应该更新状态', () => {
      const store = useProcessingStore()
      store.tasks = [{ ...mockProcessingTask, id: mockProcessingTask.task_id }]

      store.updateProgress({
        event: 'completed',
        task_id: 'task-001',
        processed_rows: 100,
        total_rows: 100,
        success_count: 95,
        error_count: 5
      })

      expect(store.tasks[0].status).toBe('completed')
    })
  })

  describe('pendingFiles 管理', () => {
    it('应该正确添加待处理文件', () => {
      const store = useProcessingStore()

      store.addPendingFile({
        id: 'file-1',
        path: '/path/to/file.xlsx',
        name: 'file.xlsx',
        size: 1024
      })

      expect(store.pendingFiles.length).toBe(1)
      expect(store.hasPendingFiles).toBe(true)
    })

    it('应该避免重复添加相同文件', () => {
      const store = useProcessingStore()

      store.addPendingFile({
        id: 'file-1',
        path: '/path/to/file.xlsx',
        name: 'file.xlsx',
        size: 1024
      })
      store.addPendingFile({
        id: 'file-2',
        path: '/path/to/file.xlsx',
        name: 'file.xlsx',
        size: 1024
      })

      expect(store.pendingFiles.length).toBe(1)
    })

    it('应该正确删除待处理文件', () => {
      const store = useProcessingStore()
      store.addPendingFile({
        id: 'file-1',
        path: '/path/to/file.xlsx',
        name: 'file.xlsx',
        size: 1024
      })

      store.removePendingFile('file-1')

      expect(store.pendingFiles.length).toBe(0)
    })

    it('应该正确清空待处理文件', () => {
      const store = useProcessingStore()
      store.addPendingFiles([
        { id: 'file-1', path: '/path/to/file1.xlsx', name: 'file1.xlsx', size: 1024 },
        { id: 'file-2', path: '/path/to/file2.xlsx', name: 'file2.xlsx', size: 2048 }
      ])

      store.clearPendingFiles()

      expect(store.pendingFiles.length).toBe(0)
    })
  })

  describe('日志管理', () => {
    it('应该正确添加日志', () => {
      const store = useProcessingStore()

      store.addLog({
        time: '10:00:00',
        message: '测试日志',
        type: 'info'
      })

      expect(store.logs.length).toBe(1)
      expect(store.logs[0].message).toBe('测试日志')
    })

    it('应该正确清空日志', () => {
      const store = useProcessingStore()
      store.addLog({ time: '10:00:00', message: '测试', type: 'info' })

      store.clearLogs()

      expect(store.logs.length).toBe(0)
    })
  })

  describe('getters', () => {
    it('activeTasks 应该只返回活动任务', () => {
      const store = useProcessingStore()
      store.tasks = [
        { ...mockProcessingTask, id: '1', status: 'processing' },
        { ...mockProcessingTask, id: '2', status: 'completed' },
        { ...mockProcessingTask, id: '3', status: 'paused' }
      ]

      expect(store.activeTasks.length).toBe(2)
    })

    it('completedTasks 应该只返回完成的任务', () => {
      const store = useProcessingStore()
      store.tasks = [
        { ...mockProcessingTask, id: '1', status: 'processing' },
        { ...mockProcessingTask, id: '2', status: 'completed' }
      ]

      expect(store.completedTasks.length).toBe(1)
    })

    it('hasActiveTasks 应该返回正确的布尔值', () => {
      const store = useProcessingStore()

      store.tasks = [{ ...mockProcessingTask, id: '1', status: 'completed' }]
      expect(store.hasActiveTasks).toBe(false)

      store.tasks = [{ ...mockProcessingTask, id: '1', status: 'processing' }]
      expect(store.hasActiveTasks).toBe(true)
    })
  })

  describe('clearTasks', () => {
    it('应该正确清空任务状态', () => {
      const store = useProcessingStore()
      store.tasks = [{ ...mockProcessingTask, id: mockProcessingTask.task_id }]
      store.activeTask = { ...mockProcessingTask, id: mockProcessingTask.task_id }
      store.progress = { event: 'test', task_id: 'test' } as any

      store.clearTasks()

      expect(store.tasks).toEqual([])
      expect(store.activeTask).toBeNull()
      expect(store.progress).toBeNull()
    })
  })
})
