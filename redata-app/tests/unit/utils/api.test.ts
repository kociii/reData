import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createMockResponse } from '../../mocks/api'
import {
  projectsApi,
  fieldsApi,
  aiConfigsApi,
  processingApi,
  resultsApi
} from '~/utils/api'
import { mockProjects, mockFields, mockAiConfigs, mockProcessingTask, mockQueryResult } from '../../fixtures/testData'

const API_BASE = 'http://127.0.0.1:8000/api'

describe('API Utils', () => {
  beforeEach(() => {
    vi.resetAllMocks()
  })

  describe('projectsApi', () => {
    describe('list', () => {
      it('应该获取项目列表', async () => {
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(mockProjects))
        global.fetch = mockFetch

        const result = await projectsApi.list()

        expect(mockFetch).toHaveBeenCalledWith(
          `${API_BASE}/projects/`,
          expect.objectContaining({ headers: expect.any(Object) })
        )
        expect(result).toEqual(mockProjects)
      })
    })

    describe('get', () => {
      it('应该获取单个项目', async () => {
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(mockProjects[0]))
        global.fetch = mockFetch

        const result = await projectsApi.get(1)

        expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/projects/1`, expect.any(Object))
        expect(result).toEqual(mockProjects[0])
      })
    })

    describe('create', () => {
      it('应该创建项目', async () => {
        const newProject = { id: 3, name: '新项目', description: '', dedup_enabled: true, dedup_fields: [], dedup_strategy: 'skip', created_at: '', updated_at: null }
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(newProject))
        global.fetch = mockFetch

        const result = await projectsApi.create({ name: '新项目' })

        expect(mockFetch).toHaveBeenCalledWith(
          `${API_BASE}/projects/`,
          expect.objectContaining({ method: 'POST' })
        )
        expect(result.name).toBe('新项目')
      })
    })

    describe('update', () => {
      it('应该更新项目', async () => {
        const updatedProject = { ...mockProjects[0], name: '更新后的名称' }
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(updatedProject))
        global.fetch = mockFetch

        const result = await projectsApi.update(1, { name: '更新后的名称' })

        expect(mockFetch).toHaveBeenCalledWith(
          `${API_BASE}/projects/1`,
          expect.objectContaining({ method: 'PUT' })
        )
        expect(result.name).toBe('更新后的名称')
      })
    })

    describe('delete', () => {
      it('应该删除项目', async () => {
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ message: '项目已删除' }))
        global.fetch = mockFetch

        await projectsApi.delete(1)

        expect(mockFetch).toHaveBeenCalledWith(
          `${API_BASE}/projects/1`,
          expect.objectContaining({ method: 'DELETE' })
        )
      })
    })
  })

  describe('fieldsApi', () => {
    describe('list', () => {
      it('应该获取字段列表', async () => {
        const projectFields = mockFields.filter(f => f.project_id === 1)
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(projectFields))
        global.fetch = mockFetch

        const result = await fieldsApi.list(1)

        expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/fields/project/1`, expect.any(Object))
        expect(result).toEqual(projectFields)
      })
    })

    describe('generateMetadata', () => {
      it('应该生成字段元数据', async () => {
        const metadata = { field_name: 'user_name', validation_rule: null, extraction_hint: '提取用户名' }
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(metadata))
        global.fetch = mockFetch

        const result = await fieldsApi.generateMetadata({
          field_label: '用户名',
          field_type: 'text'
        })

        expect(mockFetch).toHaveBeenCalledWith(
          `${API_BASE}/fields/generate-metadata`,
          expect.objectContaining({ method: 'POST' })
        )
        expect(result.field_name).toBe('user_name')
      })
    })
  })

  describe('aiConfigsApi', () => {
    describe('list', () => {
      it('应该获取配置列表', async () => {
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(mockAiConfigs))
        global.fetch = mockFetch

        const result = await aiConfigsApi.list()

        expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/ai-configs/`, expect.any(Object))
        expect(result).toEqual(mockAiConfigs)
      })
    })

    describe('getDefault', () => {
      it('应该获取默认配置', async () => {
        const defaultConfig = mockAiConfigs.find(c => c.is_default)!
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(defaultConfig))
        global.fetch = mockFetch

        const result = await aiConfigsApi.getDefault()

        expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/ai-configs/default`, expect.any(Object))
        expect(result.is_default).toBe(true)
      })
    })

    describe('testConnection', () => {
      it('应该测试连接', async () => {
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse({
          success: true,
          message: '连接成功',
          response: 'OK'
        }))
        global.fetch = mockFetch

        const result = await aiConfigsApi.testConnection(1)

        expect(mockFetch).toHaveBeenCalledWith(
          `${API_BASE}/ai-configs/test-connection`,
          expect.objectContaining({ method: 'POST' })
        )
        expect(result.success).toBe(true)
      })
    })

    describe('setDefault', () => {
      it('应该设置默认配置', async () => {
        const newDefault = { ...mockAiConfigs[1], is_default: true }
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(newDefault))
        global.fetch = mockFetch

        const result = await aiConfigsApi.setDefault(2)

        expect(mockFetch).toHaveBeenCalledWith(
          `${API_BASE}/ai-configs/2/set-default`,
          expect.objectContaining({ method: 'POST' })
        )
        expect(result.is_default).toBe(true)
      })
    })
  })

  describe('processingApi', () => {
    describe('start', () => {
      it('应该启动处理任务', async () => {
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(mockProcessingTask))
        global.fetch = mockFetch

        const result = await processingApi.start({
          project_id: 1,
          file_paths: ['/path/to/file.xlsx']
        })

        expect(mockFetch).toHaveBeenCalledWith(
          `${API_BASE}/processing/start`,
          expect.objectContaining({ method: 'POST' })
        )
        expect(result.task_id).toBe(mockProcessingTask.task_id)
      })
    })

    describe('pause/resume/cancel', () => {
      it('应该暂停任务', async () => {
        const pausedTask = { ...mockProcessingTask, status: 'paused' }
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(pausedTask))
        global.fetch = mockFetch

        const result = await processingApi.pause('task-001')

        expect(mockFetch).toHaveBeenCalledWith(
          `${API_BASE}/processing/pause/task-001`,
          expect.objectContaining({ method: 'POST' })
        )
        expect(result.status).toBe('paused')
      })

      it('应该恢复任务', async () => {
        const resumedTask = { ...mockProcessingTask, status: 'processing' }
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(resumedTask))
        global.fetch = mockFetch

        const result = await processingApi.resume('task-001')

        expect(mockFetch).toHaveBeenCalledWith(
          `${API_BASE}/processing/resume/task-001`,
          expect.objectContaining({ method: 'POST' })
        )
        expect(result.status).toBe('processing')
      })

      it('应该取消任务', async () => {
        const cancelledTask = { ...mockProcessingTask, status: 'cancelled' }
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(cancelledTask))
        global.fetch = mockFetch

        const result = await processingApi.cancel('task-001')

        expect(mockFetch).toHaveBeenCalledWith(
          `${API_BASE}/processing/cancel/task-001`,
          expect.objectContaining({ method: 'POST' })
        )
        expect(result.status).toBe('cancelled')
      })
    })

    describe('connectProgress', () => {
      it('应该创建 WebSocket 连接', () => {
        const ws = processingApi.connectProgress('task-001')

        expect(ws.url).toBe(`ws://127.0.0.1:8000/api/processing/ws/progress/task-001`)
      })
    })
  })

  describe('resultsApi', () => {
    describe('query', () => {
      it('应该查询结果', async () => {
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse(mockQueryResult))
        global.fetch = mockFetch

        const result = await resultsApi.query(1, { page: 1, page_size: 50 })

        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining(`${API_BASE}/results/1?`),
          expect.any(Object)
        )
        expect(result.records).toEqual(mockQueryResult.records)
      })
    })

    describe('update', () => {
      it('应该更新记录', async () => {
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse({}))
        global.fetch = mockFetch

        await resultsApi.update(1, 1, { name: '新名称' })

        expect(mockFetch).toHaveBeenCalledWith(
          `${API_BASE}/results/1/1`,
          expect.objectContaining({ method: 'PUT' })
        )
      })
    })

    describe('delete', () => {
      it('应该删除记录', async () => {
        const mockFetch = vi.fn().mockResolvedValue(createMockResponse({}))
        global.fetch = mockFetch

        await resultsApi.delete(1, 1)

        expect(mockFetch).toHaveBeenCalledWith(
          `${API_BASE}/results/1/1`,
          expect.objectContaining({ method: 'DELETE' })
        )
      })
    })
  })

  describe('error handling', () => {
    it('应该正确处理 API 错误', async () => {
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ detail: '错误信息' }, false, 400))
      global.fetch = mockFetch

      await expect(projectsApi.get(999)).rejects.toThrow('API Error: 400')
    })
  })
})
