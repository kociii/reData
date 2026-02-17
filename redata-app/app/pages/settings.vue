<template>
  <div>
    <!-- 页面标题 -->
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">设置</h1>
      <p class="text-gray-500 dark:text-gray-400 mt-1">配置 AI 模型和应用设置</p>
    </div>

    <!-- AI 配置 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 mb-6">
      <div class="flex justify-between items-center px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-medium text-gray-900 dark:text-white">AI 配置</h2>
        <UButton icon="i-lucide-plus" size="sm" @click="showCreateModal = true">
          添加配置
        </UButton>
      </div>

      <!-- 加载状态 -->
      <div v-if="configStore.loading" class="flex justify-center py-12">
        <UIcon name="i-lucide-refresh-cw" class="w-8 h-8 animate-spin text-primary" />
      </div>

      <!-- 空状态 -->
      <div
        v-else-if="!configStore.hasConfig"
        class="text-center py-12"
      >
        <UIcon name="i-lucide-cpu" class="w-12 h-12 mx-auto text-gray-400 mb-4" />
        <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">还没有 AI 配置</h3>
        <p class="text-gray-500 dark:text-gray-400 mb-4">添加 AI 配置以启用数据提取功能</p>
        <UButton icon="i-lucide-plus" @click="showCreateModal = true">
          添加配置
        </UButton>
      </div>

      <!-- 配置列表 -->
      <div v-else class="divide-y divide-gray-200 dark:divide-gray-700">
        <div
          v-for="config in configStore.configs"
          :key="config.id"
          class="flex items-center justify-between px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-900"
        >
          <div class="flex items-center gap-3">
            <div>
              <div class="flex items-center gap-2">
                <span class="font-medium text-gray-900 dark:text-white">{{ config.name }}</span>
                <UBadge v-if="config.is_default" color="success" variant="subtle" size="xs">
                  默认
                </UBadge>
              </div>
              <div class="text-sm text-gray-500 dark:text-gray-400">
                {{ config.model }} · {{ config.base_url || '默认端点' }}
              </div>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <UButton
              icon="i-lucide-signal"
              color="neutral"
              variant="ghost"
              size="xs"
              :loading="testingId === config.id"
              @click="testConfig(config.id)"
            >
              测试
            </UButton>
            <UButton
              v-if="!config.is_default"
              icon="i-lucide-star"
              color="neutral"
              variant="ghost"
              size="xs"
              @click="setDefaultConfig(config.id)"
            >
              设为默认
            </UButton>
            <UButton
              icon="i-lucide-square-pen"
              color="neutral"
              variant="ghost"
              size="xs"
              @click="editConfig(config)"
            >
              编辑
            </UButton>
            <UButton
              icon="i-lucide-trash-2"
              color="error"
              variant="ghost"
              size="xs"
              @click="confirmDelete(config)"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 应用设置 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-medium text-gray-900 dark:text-white">应用设置</h2>
      </div>
      <div class="p-4 space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <div class="font-medium text-gray-900 dark:text-white">深色模式</div>
            <div class="text-sm text-gray-500 dark:text-gray-400">切换应用主题</div>
          </div>
          <ColorModeButton />
        </div>

        <USeparator />

        <div class="flex items-center justify-between">
          <div>
            <div class="font-medium text-gray-900 dark:text-white">并行处理数</div>
            <div class="text-sm text-gray-500 dark:text-gray-400">同时处理的文件数量</div>
          </div>
          <UInput
            v-model="appSettings.parallelCount"
            type="number"
            min="1"
            max="10"
            class="w-24"
          />
        </div>

        <div class="flex items-center justify-between">
          <div>
            <div class="font-medium text-gray-900 dark:text-white">自动保存</div>
            <div class="text-sm text-gray-500 dark:text-gray-400">处理完成后自动保存结果</div>
          </div>
          <UToggle v-model="appSettings.autoSave" />
        </div>
      </div>
    </div>

    <!-- 创建/编辑配置对话框 -->
    <UModal v-model:open="showCreateModal" :title="editingConfig ? '编辑配置' : '添加配置'">
      <template #body>
        <div class="space-y-5 py-2">
          <UFormField label="配置名称" name="name" required>
            <UInput v-model="configForm.name" placeholder="例如：GPT-4o" class="w-full" />
          </UFormField>

          <UFormField label="模型 (Model)" name="model" required>
            <UInput v-model="configForm.model" placeholder="例如：gpt-4o, claude-3-sonnet..." class="w-full" />
          </UFormField>

          <UFormField label="Base URL" name="base_url" required>
            <UInput v-model="configForm.base_url" placeholder="https://api.openai.com/v1" class="w-full" />
          </UFormField>

          <UFormField label="API Key" name="api_key" required>
            <UInput v-model="configForm.api_key" type="password" placeholder="sk-..." class="w-full" />
          </UFormField>

          <USwitch v-model="configForm.is_default" label="设为默认配置" />
        </div>
      </template>

      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" @click="closeModal">
            取消
          </UButton>
          <UButton :loading="saving" @click="saveConfig">
            保存
          </UButton>
        </div>
      </template>
    </UModal>

    <!-- 删除确认对话框 -->
    <UModal v-model:open="showDeleteModal" title="确认删除">
      <template #body>
        <p class="text-gray-600 dark:text-gray-400">
          确定要删除配置 "{{ configToDelete?.name }}" 吗？此操作无法撤销。
        </p>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" @click="showDeleteModal = false">
            取消
          </UButton>
          <UButton color="error" :loading="deleting" @click="deleteConfig">
            删除
          </UButton>
        </div>
      </template>
    </UModal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useConfigStore } from '~/stores/config'
import type { AiConfig } from '~/types'

definePageMeta({
  layout: 'default',
})

const configStore = useConfigStore()

// 应用设置
const appSettings = reactive({
  parallelCount: 3,
  autoSave: true,
})

// 配置表单
const showCreateModal = ref(false)
const editingConfig = ref<AiConfig | null>(null)
const saving = ref(false)
const configForm = reactive({
  name: '',
  model: '',
  api_key: '',
  base_url: '',
  is_default: false,
})

// 测试连接
const testingId = ref<number | null>(null)

// 删除
const showDeleteModal = ref(false)
const configToDelete = ref<AiConfig | null>(null)
const deleting = ref(false)

// 编辑配置
function editConfig(config: AiConfig) {
  editingConfig.value = config
  configForm.name = config.name
  configForm.model = config.model || ''
  configForm.api_key = ''
  configForm.base_url = config.base_url || ''
  configForm.is_default = config.is_default
  showCreateModal.value = true
}

// 关闭对话框
function closeModal() {
  showCreateModal.value = false
  editingConfig.value = null
  configForm.name = ''
  configForm.model = ''
  configForm.api_key = ''
  configForm.base_url = ''
  configForm.is_default = false
}

// 保存配置
async function saveConfig() {
  if (!configForm.name.trim() || !configForm.model.trim()) return

  saving.value = true
  try {
    if (editingConfig.value) {
      await configStore.updateConfig(editingConfig.value.id, {
        name: configForm.name.trim(),
        model: configForm.model.trim(),
        api_key: configForm.api_key.trim() || undefined,
        base_url: configForm.base_url.trim() || undefined,
        is_default: configForm.is_default,
      })
    } else {
      await configStore.createConfig({
        name: configForm.name.trim(),
        model: configForm.model.trim(),
        api_key: configForm.api_key.trim(),
        base_url: configForm.base_url.trim() || undefined,
        is_default: configForm.is_default,
      })
    }
    closeModal()
  } catch (error) {
    console.error('Failed to save config:', error)
  } finally {
    saving.value = false
  }
}

// 测试配置
async function testConfig(id: number) {
  testingId.value = id
  try {
    const result = await configStore.testConnection(id)
    if (result.success) {
      alert('连接成功！')
    } else {
      alert(`连接失败：${result.message}`)
    }
  } catch (error) {
    console.error('Failed to test config:', error)
  } finally {
    testingId.value = null
  }
}

// 设为默认
async function setDefaultConfig(id: number) {
  try {
    await configStore.setDefault(id)
  } catch (error) {
    console.error('Failed to set default config:', error)
  }
}

// 确认删除
function confirmDelete(config: AiConfig) {
  configToDelete.value = config
  showDeleteModal.value = true
}

// 删除配置
async function deleteConfig() {
  if (!configToDelete.value) return

  deleting.value = true
  try {
    await configStore.deleteConfig(configToDelete.value.id)
    showDeleteModal.value = false
    configToDelete.value = null
  } catch (error) {
    console.error('Failed to delete config:', error)
  } finally {
    deleting.value = false
  }
}

onMounted(() => {
  configStore.fetchConfigs()
})
</script>
