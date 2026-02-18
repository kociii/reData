import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useProjectStore } from '~/stores/project'
import { mockProjects } from '../../fixtures/testData'
import { createMockResponse } from '../../mocks/api'

const API_BASE = 'http://127.0.0.1:8000/api'

describe('projectStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  describe('fetchProjects', () => {
    it('应该成功获取项目列表', async () => {
      const store = useProjectStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(mockProjects))
      global.fetch = mockFetch

      await store.fetchProjects()

      expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/projects/`, expect.any(Object))
      expect(store.projects).toEqual(mockProjects)
      expect(store.loading).toBe(false)
      expect(store.error).toBeNull()
    })

    it('应该正确处理获取失败的情况', async () => {
      const store = useProjectStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ detail: '错误' }, false, 500))
      global.fetch = mockFetch

      await store.fetchProjects()

      expect(store.projects).toEqual([])
      expect(store.error).toBe('API Error: 500 - {"detail":"错误"}')
      expect(store.loading).toBe(false)
    })

    it('应该在获取时设置 loading 状态', async () => {
      const store = useProjectStore()
      let resolveFn: Function
      const mockFetch = vi.fn().mockImplementation(() => new Promise(resolve => {
        resolveFn = resolve
      }))
      global.fetch = mockFetch

      const promise = store.fetchProjects()
      expect(store.loading).toBe(true)

      resolveFn!(createMockResponse(mockProjects))
      await promise
      expect(store.loading).toBe(false)
    })
  })

  describe('fetchProject', () => {
    it('应该成功获取单个项目', async () => {
      const store = useProjectStore()
      const mockProject = mockProjects[0]
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(mockProject))
      global.fetch = mockFetch

      await store.fetchProject(1)

      expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/projects/1`, expect.any(Object))
      expect(store.currentProject).toEqual(mockProject)
    })

    it('应该处理项目不存在的情况', async () => {
      const store = useProjectStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ detail: '项目不存在' }, false, 404))
      global.fetch = mockFetch

      await store.fetchProject(999)

      expect(store.currentProject).toBeNull()
      expect(store.error).toBeTruthy()
    })
  })

  describe('createProject', () => {
    it('应该成功创建项目', async () => {
      const store = useProjectStore()
      const newProject = {
        id: 3,
        name: '新项目',
        description: '描述',
        dedup_enabled: true,
        dedup_fields: [],
        dedup_strategy: 'skip',
        created_at: '2024-01-03T00:00:00',
        updated_at: null
      }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(newProject))
      global.fetch = mockFetch

      const result = await store.createProject({
        name: '新项目',
        description: '描述'
      })

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/projects/`,
        expect.objectContaining({ method: 'POST' })
      )
      expect(result).toEqual(newProject)
      expect(store.projects).toContainEqual(newProject)
    })

    it('应该处理创建失败的情况', async () => {
      const store = useProjectStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ detail: '项目名称已存在' }, false, 400))
      global.fetch = mockFetch

      await expect(store.createProject({ name: '已存在的项目' })).rejects.toThrow()
      expect(store.error).toBeTruthy()
    })
  })

  describe('updateProject', () => {
    it('应该成功更新项目', async () => {
      const store = useProjectStore()
      // 先设置初始数据
      store.projects = [...mockProjects]
      store.currentProject = mockProjects[0]

      const updatedProject = { ...mockProjects[0], name: '更新后的名称' }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(updatedProject))
      global.fetch = mockFetch

      const result = await store.updateProject(1, { name: '更新后的名称' })

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/projects/1`,
        expect.objectContaining({ method: 'PUT' })
      )
      expect(result.name).toBe('更新后的名称')
      expect(store.projects[0].name).toBe('更新后的名称')
      expect(store.currentProject?.name).toBe('更新后的名称')
    })
  })

  describe('deleteProject', () => {
    it('应该成功删除项目', async () => {
      const store = useProjectStore()
      store.projects = [...mockProjects]
      store.currentProject = mockProjects[0]

      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ message: '项目已删除' }))
      global.fetch = mockFetch

      await store.deleteProject(1)

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/projects/1`,
        expect.objectContaining({ method: 'DELETE' })
      )
      expect(store.projects.find(p => p.id === 1)).toBeUndefined()
      expect(store.currentProject).toBeNull()
    })
  })

  describe('setCurrentProject', () => {
    it('应该正确设置当前项目', () => {
      const store = useProjectStore()
      const project = mockProjects[0]

      store.setCurrentProject(project)
      expect(store.currentProject).toEqual(project)

      store.setCurrentProject(null)
      expect(store.currentProject).toBeNull()
    })
  })

  describe('clearError', () => {
    it('应该清除错误信息', () => {
      const store = useProjectStore()
      store.error = 'some error'

      store.clearError()
      expect(store.error).toBeNull()
    })
  })

  describe('getters', () => {
    it('projectCount 应该返回正确的项目数量', () => {
      const store = useProjectStore()
      store.projects = [...mockProjects]
      expect(store.projectCount).toBe(2)
    })

    it('hasProjects 应该返回正确的布尔值', () => {
      const store = useProjectStore()

      store.projects = []
      expect(store.hasProjects).toBe(false)

      store.projects = [...mockProjects]
      expect(store.hasProjects).toBe(true)
    })
  })
})
