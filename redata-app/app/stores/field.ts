import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ProjectField, CreateFieldRequest, FieldMetadata } from '~/types'
import { fieldsApi } from '~/utils/api'

export const useFieldStore = defineStore('field', () => {
  // State
  const fields = ref<ProjectField[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const fieldCount = computed(() => fields.value.length)
  const requiredFields = computed(() => fields.value.filter((f) => f.is_required))
  const fieldNames = computed(() => fields.value.map((f) => f.field_name))

  // Actions
  async function fetchFields(projectId: number) {
    loading.value = true
    error.value = null
    try {
      fields.value = await fieldsApi.list(projectId)
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to fetch fields:', e)
    } finally {
      loading.value = false
    }
  }

  async function createField(data: CreateFieldRequest) {
    loading.value = true
    error.value = null
    try {
      const field = await fieldsApi.create(data)
      fields.value.push(field)
      return field
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to create field:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function updateField(id: number, data: Partial<CreateFieldRequest>) {
    loading.value = true
    error.value = null
    try {
      const field = await fieldsApi.update(id, data)
      const index = fields.value.findIndex((f) => f.id === id)
      if (index !== -1) {
        fields.value[index] = field
      }
      return field
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to update field:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function deleteField(id: number) {
    loading.value = true
    error.value = null
    try {
      await fieldsApi.delete(id)
      fields.value = fields.value.filter((f) => f.id !== id)
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to delete field:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function generateMetadata(fieldLabel: string, fieldType: string): Promise<FieldMetadata> {
    loading.value = true
    error.value = null
    try {
      return await fieldsApi.generateMetadata({ field_label: fieldLabel, field_type: fieldType })
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to generate metadata:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  function clearFields() {
    fields.value = []
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    fields,
    loading,
    error,
    // Getters
    fieldCount,
    requiredFields,
    fieldNames,
    // Actions
    fetchFields,
    createField,
    updateField,
    deleteField,
    generateMetadata,
    clearFields,
    clearError,
  }
})
