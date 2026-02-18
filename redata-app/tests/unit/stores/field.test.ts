import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useFieldStore } from '~/stores/field'
import { mockFields } from '../../fixtures/testData'
import { createMockResponse } from '../../mocks/api'

const API_BASE = 'http://127.0.0.1:8000/api'

describe('fieldStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  describe('fetchFields', () => {
    it('应该成功获取字段列表', async () => {
      const store = useFieldStore()
      const projectFields = mockFields.filter(f => f.project_id === 1)
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(projectFields))
      global.fetch = mockFetch

      await store.fetchFields(1)

      expect(mockFetch).toHaveBeenCalledWith(`${API_BASE}/fields/project/1`, expect.any(Object))
      expect(store.fields).toEqual(projectFields)
      expect(store.loading).toBe(false)
    })

    it('应该处理获取失败的情况', async () => {
      const store = useFieldStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ detail: '错误' }, false, 500))
      global.fetch = mockFetch

      await store.fetchFields(1)

      expect(store.fields).toEqual([])
      expect(store.error).toBeTruthy()
    })
  })

  describe('createField', () => {
    it('应该成功创建字段', async () => {
      const store = useFieldStore()
      const newField = {
        id: 4,
        project_id: 1,
        field_name: 'address',
        field_label: '地址',
        field_type: 'text',
        is_required: false,
        is_dedup_key: false,
        additional_requirement: null,
        validation_rule: null,
        extraction_hint: null,
        display_order: 3,
        created_at: '2024-01-04T00:00:00'
      }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(newField))
      global.fetch = mockFetch

      const result = await store.createField({
        project_id: 1,
        field_name: 'address',
        field_label: '地址',
        field_type: 'text'
      })

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/fields/`,
        expect.objectContaining({ method: 'POST' })
      )
      expect(result).toEqual(newField)
      expect(store.fields).toContainEqual(newField)
    })

    it('应该处理创建失败的情况', async () => {
      const store = useFieldStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ detail: '字段名已存在' }, false, 400))
      global.fetch = mockFetch

      await expect(store.createField({
        project_id: 1,
        field_name: 'name',
        field_label: '姓名',
        field_type: 'text'
      })).rejects.toThrow()

      expect(store.error).toBeTruthy()
    })
  })

  describe('updateField', () => {
    it('应该成功更新字段', async () => {
      const store = useFieldStore()
      store.fields = [...mockFields.filter(f => f.project_id === 1)]

      const updatedField = { ...store.fields[0], field_label: '新姓名' }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(updatedField))
      global.fetch = mockFetch

      const result = await store.updateField(1, { field_label: '新姓名' })

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/fields/1`,
        expect.objectContaining({ method: 'PUT' })
      )
      expect(result.field_label).toBe('新姓名')
      expect(store.fields[0].field_label).toBe('新姓名')
    })
  })

  describe('deleteField', () => {
    it('应该成功删除字段', async () => {
      const store = useFieldStore()
      store.fields = [...mockFields.filter(f => f.project_id === 1)]
      const initialCount = store.fields.length

      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ message: '字段已删除' }))
      global.fetch = mockFetch

      await store.deleteField(1)

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/fields/1`,
        expect.objectContaining({ method: 'DELETE' })
      )
      expect(store.fields.length).toBe(initialCount - 1)
      expect(store.fields.find(f => f.id === 1)).toBeUndefined()
    })
  })

  describe('generateMetadata', () => {
    it('应该成功生成字段元数据', async () => {
      const store = useFieldStore()
      const metadata = {
        field_name: 'user_name',
        validation_rule: null,
        extraction_hint: '提取用户的全名'
      }
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse(metadata))
      global.fetch = mockFetch

      const result = await store.generateMetadata('用户名', 'text')

      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE}/fields/generate-metadata`,
        expect.objectContaining({ method: 'POST' })
      )
      expect(result.field_name).toBe('user_name')
      expect(result.extraction_hint).toBe('提取用户的全名')
    })

    it('应该处理生成失败的情况', async () => {
      const store = useFieldStore()
      const mockFetch = vi.fn().mockResolvedValue(createMockResponse({ detail: '未找到默认 AI 配置' }, false, 404))
      global.fetch = mockFetch

      await expect(store.generateMetadata('用户名', 'text')).rejects.toThrow()
      expect(store.error).toBeTruthy()
    })
  })

  describe('clearFields', () => {
    it('应该清空字段列表', () => {
      const store = useFieldStore()
      store.fields = [...mockFields]

      store.clearFields()
      expect(store.fields).toEqual([])
    })
  })

  describe('clearError', () => {
    it('应该清除错误信息', () => {
      const store = useFieldStore()
      store.error = 'some error'

      store.clearError()
      expect(store.error).toBeNull()
    })
  })

  describe('getters', () => {
    it('fieldCount 应该返回正确的字段数量', () => {
      const store = useFieldStore()
      store.fields = [...mockFields.filter(f => f.project_id === 1)]
      expect(store.fieldCount).toBe(3)
    })

    it('requiredFields 应该返回必填字段列表', () => {
      const store = useFieldStore()
      store.fields = [...mockFields.filter(f => f.project_id === 1)]
      const required = store.requiredFields
      expect(required.length).toBe(2)
      expect(required.every(f => f.is_required)).toBe(true)
    })

    it('fieldNames 应该返回字段名列表', () => {
      const store = useFieldStore()
      store.fields = [...mockFields.filter(f => f.project_id === 1)]
      expect(store.fieldNames).toEqual(['name', 'phone', 'email'])
    })
  })
})
