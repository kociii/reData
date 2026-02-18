import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useProjectStore } from '~/stores/project'
import { useFieldStore } from '~/stores/field'
import { useConfigStore } from '~/stores/config'
import { useProcessingStore } from '~/stores/processing'
import { useResultStore } from '~/stores/result'
import { createMockResponse, MockWebSocket, getWebSocketInstance, resetWebSocketInstances } from '../../mocks/api'
import { mockProjects, mockFields, mockAiConfigs, mockQueryResult } from '../../fixtures/testData'

const API_BASE = 'http://127.0.0.1:8000/api'

describe('数据导入集成测试', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    resetWebSocketInstances()

    // Mock WebSocket
    global.WebSocket = MockWebSocket as any
  })

  describe('完整数据导入流程', () => {
    it('应该能够完成整个数据导入流程', async () => {
      // ========== 步骤 1: 创建 AI 配置 ==========
      const configStore = useConfigStore()
      const mockFetch = vi.fn().mockImplementation((url: string, options?: RequestInit) => {
        const method = options?.method || 'GET'

        // 创建 AI 配置
        if (url === `${API_BASE}/ai-configs/` && method === 'POST') {
          return createMockResponse({
            id: 1,
            name: '测试配置',
            api_url: 'https://api.openai.com/v1',
            model_name: 'gpt-4',
            api_key: 'sk-test',
            is_default: true,
            created_at: new Date().toISOString()
          }, true, 201)
        }

        // 获取配置列表
        if (url === `${API_BASE}/ai-configs/` && method === 'GET') {
          return createMockResponse([{
            id: 1,
            name: '测试配置',
            api_url: 'https://api.openai.com/v1',
            model_name: 'gpt-4',
            api_key: 'sk-test',
            is_default: true,
            created_at: new Date().toISOString()
          }])
        }

        return createMockResponse({ detail: 'Not found' }, false, 404)
      })
      global.fetch = mockFetch

      await configStore.fetchConfigs()
      expect(configStore.configs.length).toBe(1)
      expect(configStore.defaultConfig).not.toBeNull()

      // ========== 步骤 2: 创建项目 ==========
      const projectStore = useProjectStore()
      mockFetch.mockImplementation((url: string, options?: RequestInit) => {
        const method = options?.method || 'GET'

        // 创建项目
        if (url === `${API_BASE}/projects/` && method === 'POST') {
          const body = JSON.parse(options?.body as string)
          return createMockResponse({
            id: 1,
            ...body,
            created_at: new Date().toISOString()
          }, true, 201)
        }

        // 获取项目列表
        if (url === `${API_BASE}/projects/` && method === 'GET') {
          return createMockResponse([{
            id: 1,
            name: '测试项目',
            description: '测试描述',
            dedup_enabled: true,
            dedup_fields: [],
            dedup_strategy: 'skip',
            created_at: new Date().toISOString()
          }])
        }

        return createMockResponse({ detail: 'Not found' }, false, 404)
      })

      await projectStore.createProject({
        name: '测试项目',
        description: '测试描述'
      })
      expect(projectStore.projects.length).toBe(1)
      const project = projectStore.projects[0]

      // ========== 步骤 3: 添加字段定义 ==========
      const fieldStore = useFieldStore()
      mockFetch.mockImplementation((url: string, options?: RequestInit) => {
        const method = options?.method || 'GET'

        // 创建字段
        if (url === `${API_BASE}/fields/` && method === 'POST') {
          const body = JSON.parse(options?.body as string)
          return createMockResponse({
            id: 1,
            project_id: project.id,
            ...body,
            created_at: new Date().toISOString()
          }, true, 201)
        }

        // 获取字段列表
        if (url.match(/\/fields\/project\/\d+$/) && method === 'GET') {
          return createMockResponse([
            {
              id: 1,
              project_id: project.id,
              field_name: 'name',
              field_label: '姓名',
              field_type: 'text',
              is_required: true,
              is_dedup_key: false,
              created_at: new Date().toISOString()
            }
          ])
        }

        return createMockResponse({ detail: 'Not found' }, false, 404)
      })

      await fieldStore.fetchFields(project.id)
      expect(fieldStore.fields.length).toBe(1)

      // ========== 步骤 4: 启动数据处理 ==========
      const processingStore = useProcessingStore()
      mockFetch.mockImplementation((url: string, options?: RequestInit) => {
        const method = options?.method || 'GET'

        // 启动处理
        if (url === `${API_BASE}/processing/start` && method === 'POST') {
          return createMockResponse({
            task_id: 'test-task-001',
            project_id: project.id,
            status: 'processing',
            total_files: 1,
            processed_files: 0,
            total_rows: 100,
            processed_rows: 0,
            success_count: 0,
            error_count: 0,
            batch_number: 'batch_001',
            created_at: new Date().toISOString()
          })
        }

        // 获取任务列表
        if (url.match(/\/processing\/list\/\d+\?/) && method === 'GET') {
          return createMockResponse({
            tasks: [{
              task_id: 'test-task-001',
              project_id: project.id,
              status: 'processing',
              total_files: 1,
              processed_files: 0,
              total_rows: 100,
              processed_rows: 50,
              success_count: 48,
              error_count: 2,
              batch_number: 'batch_001'
            }],
            total: 1
          })
        }

        return createMockResponse({ detail: 'Not found' }, false, 404)
      })

      const task = await processingStore.startProcessing(project.id, ['/test/data.xlsx'])
      expect(task).not.toBeNull()
      expect(task.status).toBe('processing')

      // ========== 步骤 5: 模拟 WebSocket 进度更新 ==========
      // 模拟连接 WebSocket
      processingStore.connectWebSocket(task.task_id)

      // 获取 WebSocket 实例并模拟消息
      const ws = getWebSocketInstance(task.task_id)
      expect(ws).not.toBeUndefined()

      // 模拟接收进度消息
      ws.simulateMessage({
        event: 'row_processed',
        task_id: task.task_id,
        processed_rows: 50,
        total_rows: 100,
        success_count: 48,
        error_count: 2,
        message: '处理进度: 50/100'
      })

      // 验证进度更新
      expect(processingStore.progress).not.toBeNull()
      expect(processingStore.progress?.processed_rows).toBe(50)

      // 模拟任务完成
      ws.simulateMessage({
        event: 'completed',
        task_id: task.task_id,
        processed_rows: 100,
        total_rows: 100,
        success_count: 98,
        error_count: 2
      })

      // 验证任务状态更新
      expect(processingStore.tasks[0].status).toBe('completed')

      // ========== 步骤 6: 查询处理结果 ==========
      const resultStore = useResultStore()
      mockFetch.mockImplementation((url: string, options?: RequestInit) => {
        const method = options?.method || 'GET'

        // 查询结果
        if (url.match(/\/results\/\d+\?/) && method === 'GET') {
          return createMockResponse({
            records: [
              { id: 1, name: '张三', phone: '13800138001', status: 'success' },
              { id: 2, name: '李四', phone: '13800138002', status: 'success' }
            ],
            total: 2,
            page: 1,
            page_size: 50
          })
        }

        return createMockResponse({ detail: 'Not found' }, false, 404)
      })

      await resultStore.fetchRecords(project.id)
      expect(resultStore.records.length).toBe(2)
      expect(resultStore.totalCount).toBe(2)

      // ========== 步骤 7: 更新记录 ==========
      mockFetch.mockImplementation((url: string, options?: RequestInit) => {
        const method = options?.method || 'GET'

        // 更新记录
        if (url.match(/\/results\/\d+\/\d+$/) && method === 'PUT') {
          return createMockResponse({ id: 1, name: '王五', phone: '13800138001', status: 'success' })
        }

        return createMockResponse({ detail: 'Not found' }, false, 404)
      })

      await resultStore.updateRecord(project.id, 1, { name: '王五' })
      // 查找更新后的记录
      const updatedRecord = resultStore.records.find(r => r.id === 1)
      expect(updatedRecord?.name).toBe('王五')

      // ========== 步骤 8: 删除记录 ==========
      const initialCount = resultStore.records.length
      mockFetch.mockImplementation((url: string, options?: RequestInit) => {
        const method = options?.method || 'GET'

        // 删除记录
        if (url.match(/\/results\/\d+\/\d+$/) && method === 'DELETE') {
          return createMockResponse({})
        }

        return createMockResponse({ detail: 'Not found' }, false, 404)
      })

      await resultStore.deleteRecord(project.id, 1)
      expect(resultStore.records.length).toBe(initialCount - 1)
    })

    it('应该正确处理 WebSocket 错误', () => {
      const processingStore = useProcessingStore()

      // 连接 WebSocket
      processingStore.connectWebSocket('test-task-error')

      const ws = getWebSocketInstance('test-task-error')
      expect(ws).not.toBeUndefined()

      // 验证初始日志为空
      expect(processingStore.logs.length).toBe(0)

      // 模拟 WebSocket 打开
      ws.simulateOpen()

      // 验证日志
      expect(processingStore.logs.length).toBe(1)
      expect(processingStore.logs[0].type).toBe('info')
    })

    it('应该正确处理暂停和恢复任务', async () => {
      const processingStore = useProcessingStore()
      const mockFetch = vi.fn().mockImplementation((url: string, options?: RequestInit) => {
        const method = options?.method || 'GET'

        // 启动处理
        if (url === `${API_BASE}/processing/start` && method === 'POST') {
          return createMockResponse({
            task_id: 'test-task-pause',
            project_id: 1,
            status: 'processing',
            total_rows: 100,
            processed_rows: 30,
            success_count: 28,
            error_count: 2,
            created_at: new Date().toISOString()
          })
        }

        // 暂停任务
        if (url.match(/\/processing\/pause\/[\w-]+$/) && method === 'POST') {
          return createMockResponse({
            task_id: 'test-task-pause',
            project_id: 1,
            status: 'paused',
            total_rows: 100,
            processed_rows: 30,
            success_count: 28,
            error_count: 2,
            updated_at: new Date().toISOString()
          })
        }

        // 恢复任务
        if (url.match(/\/processing\/resume\/[\w-]+$/) && method === 'POST') {
          return createMockResponse({
            task_id: 'test-task-pause',
            project_id: 1,
            status: 'processing',
            total_rows: 100,
            processed_rows: 30,
            success_count: 28,
            error_count: 2,
            updated_at: new Date().toISOString()
          })
        }

        return createMockResponse({ detail: 'Not found' }, false, 404)
      })
      global.fetch = mockFetch

      // 启动任务
      const task = await processingStore.startProcessing(1, ['/test/data.xlsx'])
      expect(task.status).toBe('processing')

      // 暂停任务
      await processingStore.pauseTask(task.task_id)
      expect(processingStore.tasks[0].status).toBe('paused')

      // 恢复任务
      await processingStore.resumeTask(task.task_id)
      expect(processingStore.tasks[0].status).toBe('processing')
    })

    it('应该正确处理取消任务', async () => {
      const processingStore = useProcessingStore()
      const mockFetch = vi.fn().mockImplementation((url: string, options?: RequestInit) => {
        const method = options?.method || 'GET'

        // 启动处理
        if (url === `${API_BASE}/processing/start` && method === 'POST') {
          return createMockResponse({
            task_id: 'test-task-cancel',
            project_id: 1,
            status: 'processing',
            total_rows: 100,
            processed_rows: 20,
            success_count: 18,
            error_count: 2,
            created_at: new Date().toISOString()
          })
        }

        // 取消任务
        if (url.match(/\/processing\/cancel\/[\w-]+$/) && method === 'POST') {
          return createMockResponse({
            task_id: 'test-task-cancel',
            project_id: 1,
            status: 'cancelled',
            total_rows: 100,
            processed_rows: 20,
            success_count: 18,
            error_count: 2,
            updated_at: new Date().toISOString()
          })
        }

        return createMockResponse({ detail: 'Not found' }, false, 404)
      })
      global.fetch = mockFetch

      // 启动任务
      const task = await processingStore.startProcessing(1, ['/test/data.xlsx'])
      expect(task.status).toBe('processing')

      // 取消任务
      await processingStore.cancelTask(task.task_id)
      expect(processingStore.tasks[0].status).toBe('cancelled')
    })

    it('应该正确处理文件完成事件', () => {
      const processingStore = useProcessingStore()

      // 连接 WebSocket
      processingStore.connectWebSocket('test-task-file')

      const ws = getWebSocketInstance('test-task-file')
      // 触发连接打开
      ws.simulateOpen()

      // 模拟文件完成消息
      ws.simulateMessage({
        event: 'file_completed',
        task_id: 'test-task-file',
        file_name: 'test.xlsx',
        sheet_name: 'Sheet1',
        row_count: 50
      })

      // 验证日志
      expect(processingStore.logs.length).toBeGreaterThanOrEqual(1)
      const fileLog = processingStore.logs.find(log => log.message?.includes('test.xlsx'))
      expect(fileLog).toBeDefined()
    })

    it('应该正确处理错误事件', () => {
      const processingStore = useProcessingStore()

      // 连接 WebSocket
      processingStore.connectWebSocket('test-task-error-event')

      const ws = getWebSocketInstance('test-task-error-event')
      // 触发连接打开
      ws.simulateOpen()

      // 模拟错误消息
      ws.simulateMessage({
        event: 'error',
        task_id: 'test-task-error-event',
        message: '文件读取失败'
      })

      // 验证日志
      const errorLog = processingStore.logs.find(log => log.type === 'error')
      expect(errorLog).toBeDefined()
      expect(errorLog?.message).toContain('文件读取失败')
    })

    it('应该正确处理暂停和恢复事件', () => {
      const processingStore = useProcessingStore()

      // 连接 WebSocket
      processingStore.connectWebSocket('test-task-paused-event')

      const ws = getWebSocketInstance('test-task-paused-event')
      // 触发连接打开
      ws.simulateOpen()

      // 模拟暂停事件
      ws.simulateMessage({
        event: 'paused',
        task_id: 'test-task-paused-event'
      })

      const pausedLog = processingStore.logs.find(log => log.message?.includes('已暂停'))
      expect(pausedLog).toBeDefined()

      // 模拟恢复事件
      ws.simulateMessage({
        event: 'resumed',
        task_id: 'test-task-paused-event'
      })

      const resumedLog = processingStore.logs.find(log => log.message?.includes('已恢复'))
      expect(resumedLog).toBeDefined()
    })
  })

  describe('WebSocket 连接管理', () => {
    it('应该能够断开所有 WebSocket 连接', () => {
      const processingStore = useProcessingStore()

      // 创建多个连接
      processingStore.connectWebSocket('task-1')
      processingStore.connectWebSocket('task-2')
      processingStore.connectWebSocket('task-3')

      expect(processingStore.wsConnections.size).toBe(3)

      // 断开所有连接
      processingStore.disconnectAllWebSockets()

      expect(processingStore.wsConnections.size).toBe(0)
    })

    it('应该能够单独断开指定连接', () => {
      const processingStore = useProcessingStore()

      processingStore.connectWebSocket('task-1')
      processingStore.connectWebSocket('task-2')

      expect(processingStore.wsConnections.size).toBe(2)

      processingStore.disconnectWebSocket('task-1')

      expect(processingStore.wsConnections.size).toBe(1)
      expect(processingStore.wsConnections.has('task-1')).toBe(false)
      expect(processingStore.wsConnections.has('task-2')).toBe(true)
    })

    it('断开重复连接时应该先断开旧的', () => {
      const processingStore = useProcessingStore()

      // 连接同一个任务两次
      processingStore.connectWebSocket('task-1')
      const ws1 = getWebSocketInstance('task-1')

      processingStore.connectWebSocket('task-1')
      const ws2 = getWebSocketInstance('task-1')

      // 应该只有一个连接
      expect(processingStore.wsConnections.size).toBe(1)
      // 新的连接应该替换旧的
      expect(ws1).not.toBe(ws2)
    })
  })

  describe('日志管理', () => {
    it('应该限制日志数量在 200 条以内', () => {
      const processingStore = useProcessingStore()

      // 添加 250 条日志
      for (let i = 0; i < 250; i++) {
        processingStore.addLog({
          time: '00:00:00',
          message: `日志 ${i}`,
          type: 'info'
        })
      }

      expect(processingStore.logs.length).toBe(200)
      expect(processingStore.logs[0].message).toBe('日志 50') // 最早的日志被移除
    })

    it('应该能够清空日志', () => {
      const processingStore = useProcessingStore()

      processingStore.addLog({ time: '00:00:00', message: '测试', type: 'info' })
      expect(processingStore.logs.length).toBe(1)

      processingStore.clearLogs()
      expect(processingStore.logs.length).toBe(0)
    })
  })
})
