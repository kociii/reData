import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { Store } from '@tauri-apps/plugin-store'

// 全局设置类型
export interface GlobalSettings {
  // 并行处理数（1-10）
  parallelCount: number
  // 主题设置：'light' | 'dark' | 'system'
  theme: 'light' | 'dark' | 'system'
  // 自动保存
  autoSave: boolean
  // 重复数据处理策略：'skip' | 'update' | 'merge'
  duplicateStrategy: 'skip' | 'update' | 'merge'
}

// 默认设置
const defaultSettings: GlobalSettings = {
  parallelCount: 3,
  theme: 'system',
  autoSave: true,
  duplicateStrategy: 'skip',
}

// Store 文件路径
const STORE_PATH = 'settings.json'

export const useGlobalSettingsStore = defineStore('globalSettings', () => {
  const settings = ref<GlobalSettings>({ ...defaultSettings })
  const loading = ref(false)
  const saving = ref(false)
  const error = ref<string | null>(null)

  let store: Store | null = null
  let colorMode: ReturnType<typeof useColorMode> | null = null

  // 初始化颜色模式同步
  function initColorMode() {
    if (colorMode) return
    colorMode = useColorMode()

    // 监听 Nuxt 颜色模式变化，同步到设置
    watch(
      () => colorMode?.preference,
      (newPreference) => {
        if (newPreference && newPreference !== settings.value.theme) {
          settings.value.theme = newPreference as GlobalSettings['theme']
        }
      },
      { immediate: true }
    )
  }

  // 加载设置
  async function loadSettings() {
    loading.value = true
    error.value = null
    try {
      store = new Store(STORE_PATH)
      await store.load()

      // 读取设置
      const parallelCount = await store.get<number>('parallelCount')
      const theme = await store.get<'light' | 'dark' | 'system'>('theme')
      const autoSave = await store.get<boolean>('autoSave')
      const duplicateStrategy = await store.get<'skip' | 'update' | 'merge'>('duplicateStrategy')

      settings.value = {
        parallelCount: parallelCount ?? defaultSettings.parallelCount,
        theme: theme ?? defaultSettings.theme,
        autoSave: autoSave ?? defaultSettings.autoSave,
        duplicateStrategy: duplicateStrategy ?? defaultSettings.duplicateStrategy,
      }

      // 同步到 Nuxt 颜色模式
      initColorMode()
      if (colorMode && settings.value.theme) {
        colorMode.preference = settings.value.theme
      }

      console.log('[GlobalSettings] Loaded settings:', settings.value)
    } catch (e) {
      console.error('[GlobalSettings] Failed to load settings:', e)
      error.value = String(e)
      // 使用默认设置
      settings.value = { ...defaultSettings }
    } finally {
      loading.value = false
    }
  }

  // 保存设置
  async function saveSettings() {
    if (!store) {
      store = new Store(STORE_PATH)
      await store.load()
    }

    saving.value = true
    error.value = null
    try {
      await store.set('parallelCount', settings.value.parallelCount)
      await store.set('theme', settings.value.theme)
      await store.set('autoSave', settings.value.autoSave)
      await store.set('duplicateStrategy', settings.value.duplicateStrategy)
      await store.save()

      console.log('[GlobalSettings] Saved settings:', settings.value)
    } catch (e) {
      console.error('[GlobalSettings] Failed to save settings:', e)
      error.value = String(e)
    } finally {
      saving.value = false
    }
  }

  // 更新单个设置
  async function updateSetting<K extends keyof GlobalSettings>(
    key: K,
    value: GlobalSettings[K]
  ) {
    settings.value[key] = value

    // 如果是主题设置，同步到 Nuxt 颜色模式
    if (key === 'theme') {
      initColorMode()
      if (colorMode) {
        colorMode.preference = value as 'light' | 'dark' | 'system'
      }
    }

    await saveSettings()
  }

  // 重置为默认设置
  async function resetSettings() {
    settings.value = { ...defaultSettings }

    // 重置主题
    initColorMode()
    if (colorMode) {
      colorMode.preference = defaultSettings.theme
    }

    await saveSettings()
  }

  // 监听设置变化并自动保存
  watch(
    settings,
    () => {
      if (!loading.value) {
        saveSettings()
      }
    },
    { deep: true }
  )

  return {
    settings,
    loading,
    saving,
    error,
    loadSettings,
    saveSettings,
    updateSetting,
    resetSettings,
  }
})
