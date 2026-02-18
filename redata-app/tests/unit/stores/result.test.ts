import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useResultStore } from '~/stores/result'
import { mockQueryResult } from '../../fixtures/testData'
import { createMockResponse } from '../../mocks/api'

const API_BASE = 'http://127.0.0.1:8000/api'

describe('resultStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  describe('fetchRecords', () => {
    it('应该成功获取记录列表', async () => {
      const store = useResultStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(mockQueryResult))
      global.fetch = mockFetch

      await store.fetchRecords(1)

      expect(store.records).toEqual(mockQueryResult.records)
      expect(store.totalCount).toBe(mockQueryResult.total)
      expect(store.currentPage).toBe(1)
      expect(store.loading).toBe(false)
    })

    it('应该正确处理分页', async () => {
      const store = useResultStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(mockQueryResult))
      global.fetch = mockFetch

      await store.fetchRecords(1, 2)

      expect(store.currentPage).toBe(2)
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('page=2'),
        expect.any(Object)
      )
    })

    it('应该正确处理搜索查询', async () => {
      const store = useResultStore()
      store.searchQuery = '张三'
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(mockQueryResult))
      global.fetch = mockFetch

      await store.fetchRecords(1)

      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('search=%E5%BC%A0%E4%B8%89'),
        expect.any(Object)
      )
    })

    it('应该处理获取失败的情况', async () => {
      const store = useResultStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ detail: '错误' }, false, 500))
      global.fetch = mockFetch

      await store.fetchRecords(1)

      expect(store.records).toEqual([])
      expect(store.error).toBeTruthy()
    })
  })

  describe('updateRecord', () => {
    it('应该成功更新记录', async () => {
      const store = useResultStore()
      store.records = [...mockQueryResult.records]

      const updatedRecord = { ...mockQueryResult.records[0], name: '王五' }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(updatedRecord))
      global.fetch = mockFetch

      const result = await store.updateRecord(1, 1, { name: '王五' })

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/results/1/1`,
        expect.objectContaining({ method: 'PUT' })
      )
      expect(store.records[0].name).toBe('王五')
    })

    it('应该处理更新失败的情况', async () => {
      const store = useResultStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ detail: '记录不存在' }, false, 404))
      global.fetch = mockFetch

      await expect(store.updateRecord(1, 999, { name: '测试' })).rejects.toThrow()
      expect(store.error).toBeTruthy()
    })
  })

  describe('deleteRecord', () => {
    it('应该成功删除记录', async () => {
      const store = useResultStore()
      store.records = [...mockQueryResult.records]
      store.totalCount = mockQueryResult.total
      const initialCount = store.records.length

      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({}))
      global.fetch = mockFetch

      await store.deleteRecord(1, 1)

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/results/1/1`,
        expect.objectContaining({ method: 'DELETE' })
      )
      expect(store.records.length).toBe(initialCount - 1)
      expect(store.totalCount).toBe(mockQueryResult.total - 1)
    })
  })

  describe('setSearchQuery', () => {
    it('应该正确设置搜索查询', () => {
      const store = useResultStore()

      store.setSearchQuery('测试搜索')

      expect(store.searchQuery).toBe('测试搜索')
    })
  })

  describe('setPageSize', () => {
    it('应该正确设置每页数量', () => {
      const store = useResultStore()

      store.setPageSize(50)

      expect(store.pageSize).toBe(50)
    })
  })

  describe('clearRecords', () => {
    it('应该正确清空记录状态', () => {
      const store = useResultStore()
      store.records = [...mockQueryResult.records]
      store.totalCount = mockQueryResult.total
      store.currentPage = 5

      store.clearRecords()

      expect(store.records).toEqual([])
      expect(store.totalCount).toBe(0)
      expect(store.currentPage).toBe(1)
    })
  })

  describe('clearError', () => {
    it('应该清除错误信息', () => {
      const store = useResultStore()
      store.error = 'some error'

      store.clearError()
      expect(store.error).toBeNull()
    })
  })

  describe('getters', () => {
    it('totalPages 应该返回正确的页数', () => {
      const store = useResultStore()
      store.totalCount = 100
      store.pageSize = 20

      expect(store.totalPages).toBe(5)
    })

    it('totalPages 应该向上取整', () => {
      const store = useResultStore()
      store.totalCount = 101
      store.pageSize = 20

      expect(store.totalPages).toBe(6)
    })

    it('hasRecords 应该返回正确的布尔值', () => {
      const store = useResultStore()

      store.records = []
      expect(store.hasRecords).toBe(false)

      store.records = [...mockQueryResult.records]
      expect(store.hasRecords).toBe(true)
    })
  })
})
