import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { AiConfig, CreateAiConfigRequest } from '~/types'
import { aiConfigsApi } from '~/utils/api'

export const useConfigStore = defineStore('config', () => {
  // State
  const configs = ref<AiConfig[]>([])
  const defaultConfig = ref<AiConfig | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const configCount = computed(() => configs.value.length)
  const hasConfig = computed(() => configs.value.length > 0)

  // Actions
  async function fetchConfigs() {
    loading.value = true
    error.value = null
    try {
      configs.value = await aiConfigsApi.list()
      defaultConfig.value = configs.value.find((c) => c.is_default) || null
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to fetch configs:', e)
    } finally {
      loading.value = false
    }
  }

  async function fetchDefaultConfig() {
    loading.value = true
    error.value = null
    try {
      defaultConfig.value = await aiConfigsApi.getDefault()
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to fetch default config:', e)
    } finally {
      loading.value = false
    }
  }

  async function createConfig(data: CreateAiConfigRequest) {
    loading.value = true
    error.value = null
    try {
      const config = await aiConfigsApi.create(data)
      configs.value.push(config)
      if (config.is_default) {
        defaultConfig.value = config
        // 更新其他配置的默认状态
        configs.value = configs.value.map((c) =>
          c.id === config.id ? c : { ...c, is_default: false }
        )
      }
      return config
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to create config:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function updateConfig(id: number, data: Partial<CreateAiConfigRequest>) {
    loading.value = true
    error.value = null
    try {
      const config = await aiConfigsApi.update(id, data)
      const index = configs.value.findIndex((c) => c.id === id)
      if (index !== -1) {
        configs.value[index] = config
      }
      if (config.is_default) {
        defaultConfig.value = config
      } else if (defaultConfig.value?.id === id) {
        defaultConfig.value = null
      }
      return config
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to update config:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function deleteConfig(id: number) {
    loading.value = true
    error.value = null
    try {
      await aiConfigsApi.delete(id)
      configs.value = configs.value.filter((c) => c.id !== id)
      if (defaultConfig.value?.id === id) {
        defaultConfig.value = configs.value.find((c) => c.is_default) || null
      }
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to delete config:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function testConnection(id: number): Promise<{ success: boolean; message: string }> {
    loading.value = true
    error.value = null
    try {
      return await aiConfigsApi.testConnection(id)
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to test connection:', e)
      return { success: false, message: e.message }
    } finally {
      loading.value = false
    }
  }

  async function setDefault(id: number) {
    loading.value = true
    error.value = null
    try {
      const config = await aiConfigsApi.setDefault(id)
      defaultConfig.value = config
      configs.value = configs.value.map((c) =>
        c.id === id ? { ...c, is_default: true } : { ...c, is_default: false }
      )
      return config
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to set default config:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    configs,
    defaultConfig,
    loading,
    error,
    // Getters
    configCount,
    hasConfig,
    // Actions
    fetchConfigs,
    fetchDefaultConfig,
    createConfig,
    updateConfig,
    deleteConfig,
    testConnection,
    setDefault,
    clearError,
  }
})
