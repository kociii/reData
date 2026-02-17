import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { QueryResult } from '~/types'
import { resultsApi } from '~/utils/api'

export const useResultStore = defineStore('result', () => {
  // State
  const records = ref<QueryResult['records']>([])
  const totalCount = ref(0)
  const currentPage = ref(1)
  const pageSize = ref(20)
  const searchQuery = ref('')
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const totalPages = computed(() => Math.ceil(totalCount.value / pageSize.value))
  const hasRecords = computed(() => records.value.length > 0)

  // Actions
  async function fetchRecords(projectId: number, page = 1) {
    loading.value = true
    error.value = null
    try {
      currentPage.value = page
      const result = await resultsApi.query(projectId, {
        page,
        page_size: pageSize.value,
        search: searchQuery.value || undefined,
      })
      records.value = result.records
      totalCount.value = result.total
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to fetch records:', e)
    } finally {
      loading.value = false
    }
  }

  async function updateRecord(projectId: number, recordId: number, data: Record<string, any>) {
    loading.value = true
    error.value = null
    try {
      const updated = await resultsApi.update(projectId, recordId, data)
      const index = records.value.findIndex(r => r.id === recordId)
      if (index !== -1) {
        records.value[index] = updated
      }
      return updated
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to update record:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function deleteRecord(projectId: number, recordId: number) {
    loading.value = true
    error.value = null
    try {
      await resultsApi.delete(projectId, recordId)
      records.value = records.value.filter(r => r.id !== recordId)
      totalCount.value--
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to delete record:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function exportData(projectId: number, format: 'xlsx' | 'csv' | 'json' = 'xlsx') {
    loading.value = true
    error.value = null
    try {
      const blob = await resultsApi.export(projectId, format)
      // 创建下载链接
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `export_${projectId}.${format}`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      URL.revokeObjectURL(url)
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to export data:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  function setSearchQuery(query: string) {
    searchQuery.value = query
  }

  function setPageSize(size: number) {
    pageSize.value = size
  }

  function clearRecords() {
    records.value = []
    totalCount.value = 0
    currentPage.value = 1
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    records,
    totalCount,
    currentPage,
    pageSize,
    searchQuery,
    loading,
    error,
    // Getters
    totalPages,
    hasRecords,
    // Actions
    fetchRecords,
    updateRecord,
    deleteRecord,
    exportData,
    setSearchQuery,
    setPageSize,
    clearRecords,
    clearError,
  }
})
