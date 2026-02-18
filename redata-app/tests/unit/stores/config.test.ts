import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useConfigStore } from '~/stores/config'
import { mockAiConfigs } from '../../fixtures/testData'
import { createMockResponse } from '../../mocks/api'

const API_BASE = 'http://127.0.0.1:8000/api'

describe('configStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  describe('fetchConfigs', () => {
    it('应该成功获取配置列表并设置默认配置', async () => {
      const store = useConfigStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(mockAiConfigs))
      global.fetch = mockFetch

      await store.fetchConfigs()

      expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/ai-configs/`, expect.any(Object))
      expect(store.configs).toEqual(mockAiConfigs)
      expect(store.defaultConfig).toEqual(mockAiConfigs[0]) // 第一个是默认的
      expect(store.loading).toBe(false)
    })

    it('应该处理获取失败的情况', async () => {
      const store = useConfigStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ detail: '错误' }, false, 500))
      global.fetch = mockFetch

      await store.fetchConfigs()

      expect(store.configs).toEqual([])
      expect(store.error).toBeTruthy()
    })
  })

  describe('fetchDefaultConfig', () => {
    it('应该成功获取默认配置', async () => {
      const store = useConfigStore()
      const defaultConfig = mockAiConfigs.find(c => c.is_default)!
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(defaultConfig))
      global.fetch = mockFetch

      await store.fetchDefaultConfig()

      expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/ai-configs/default`, expect.any(Object))
      expect(store.defaultConfig).toEqual(defaultConfig)
    })

    it('应该处理没有默认配置的情况', async () => {
      const store = useConfigStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ detail: '未找到默认配置' }, false, 404))
      global.fetch = mockFetch

      await store.fetchDefaultConfig()

      expect(store.error).toBeTruthy()
    })
  })

  describe('createConfig', () => {
    it('应该成功创建配置', async () => {
      const store = useConfigStore()
      const newConfig = {
        id: 3,
        name: '新配置',
        api_url: 'https://api.example.com/v1',
        model_name: 'model-1',
        api_key: 'test-key',
        temperature: 0.5,
        max_tokens: 2000,
        is_default: false,
        created_at: '2024-01-03T00:00:00',
        updated_at: null
      }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(newConfig))
      global.fetch = mockFetch

      const result = await store.createConfig({
        name: '新配置',
        api_url: 'https://api.example.com/v1',
        model_name: 'model-1',
        api_key: 'test-key'
      })

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/ai-configs/`,
        expect.objectContaining({ method: 'POST' })
      )
      expect(result).toEqual(newConfig)
      expect(store.configs).toContainEqual(newConfig)
    })

    it('创建默认配置时应该更新其他配置的默认状态', async () => {
      const store = useConfigStore()
      store.configs = [...mockAiConfigs]

      const newDefaultConfig = {
        id: 3,
        name: '新默认配置',
        api_url: 'https://api.example.com/v1',
        model_name: 'model-1',
        api_key: 'test-key',
        temperature: 0.5,
        max_tokens: 2000,
        is_default: true,
        created_at: '2024-01-03T00:00:00',
        updated_at: null
      }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(newDefaultConfig))
      global.fetch = mockFetch

      await store.createConfig({
        name: '新默认配置',
        api_url: 'https://api.example.com/v1',
        model_name: 'model-1',
        api_key: 'test-key',
        is_default: true
      })

      expect(store.defaultConfig).toEqual(newDefaultConfig)
      // 其他配置应该不再是默认的
      expect(store.configs.filter(c => c.is_default).length).toBe(1)
    })
  })

  describe('updateConfig', () => {
    it('应该成功更新配置', async () => {
      const store = useConfigStore()
      store.configs = [...mockAiConfigs]

      const updatedConfig = { ...mockAiConfigs[0], name: '更新后的名称' }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(updatedConfig))
      global.fetch = mockFetch

      const result = await store.updateConfig(1, { name: '更新后的名称' })

      expect(result.name).toBe('更新后的名称')
      expect(store.configs[0].name).toBe('更新后的名称')
    })
  })

  describe('deleteConfig', () => {
    it('应该成功删除配置', async () => {
      const store = useConfigStore()
      store.configs = [...mockAiConfigs]
      const initialCount = store.configs.length

      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({}))
      global.fetch = mockFetch

      await store.deleteConfig(2)

      expect(store.configs.length).toBe(initialCount - 1)
      expect(store.configs.find(c => c.id === 2)).toBeUndefined()
    })

    it('删除默认配置时应该重新选择默认配置', async () => {
      const store = useConfigStore()
      store.configs = [...mockAiConfigs]
      store.defaultConfig = mockAiConfigs[0]

      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({}))
      global.fetch = mockFetch

      await store.deleteConfig(1)

      // 删除后应该重新选择默认配置或为 null
      expect(store.configs.find(c => c.id === 1)).toBeUndefined()
    })
  })

  describe('testConnection', () => {
    it('应该成功测试连接', async () => {
      const store = useConfigStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({
        success: true,
        message: '连接成功',
        response: 'OK'
      }))
      global.fetch = mockFetch

      const result = await store.testConnection(1)

      expect(result.success).toBe(true)
      expect(result.message).toBe('连接成功')
    })

    it('应该处理连接失败的情况', async () => {
      const store = useConfigStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ detail: '连接失败' }, false, 500))
      global.fetch = mockFetch

      const result = await store.testConnection(1)

      expect(result.success).toBe(false)
    })
  })

  describe('setDefault', () => {
    it('应该成功设置默认配置', async () => {
      const store = useConfigStore()
      store.configs = [...mockAiConfigs]

      const newDefault = { ...mockAiConfigs[1], is_default: true }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(newDefault))
      global.fetch = mockFetch

      await store.setDefault(2)

      expect(store.defaultConfig?.id).toBe(2)
      expect(store.configs.find(c => c.id === 2)?.is_default).toBe(true)
      expect(store.configs.find(c => c.id === 1)?.is_default).toBe(false)
    })
  })

  describe('clearError', () => {
    it('应该清除错误信息', () => {
      const store = useConfigStore()
      store.error = 'some error'

      store.clearError()
      expect(store.error).toBeNull()
    })
  })

  describe('getters', () => {
    it('configCount 应该返回正确的配置数量', () => {
      const store = useConfigStore()
      store.configs = [...mockAiConfigs]
      expect(store.configCount).toBe(2)
    })

    it('hasConfig 应该返回正确的布尔值', () => {
      const store = useConfigStore()

      store.configs = []
      expect(store.hasConfig).toBe(false)

      store.configs = [...mockAiConfigs]
      expect(store.hasConfig).toBe(true)
    })
  })
})
