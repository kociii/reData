<template>
  <div class="h-full overflow-auto p-6">
    <!-- 页面标题 -->
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-highlighted">设置</h1>
      <p class="text-muted mt-1">配置 AI 模型和应用设置</p>
    </div>

    <!-- AI 配置 -->
    <div class="bg-elevated rounded-lg border border-default mb-6">
      <div class="flex justify-between items-center px-4 py-3 border-b border-default">
        <h2 class="text-lg font-medium text-highlighted">AI 配置</h2>
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
        <UIcon name="i-lucide-cpu" class="w-12 h-12 mx-auto text-dimmed mb-4" />
        <h3 class="text-lg font-medium text-highlighted mb-2">还没有 AI 配置</h3>
        <p class="text-muted mb-4">添加 AI 配置以启用数据提取功能</p>
        <UButton icon="i-lucide-plus" @click="showCreateModal = true">
          添加配置
        </UButton>
      </div>

      <!-- 配置列表 -->
      <div v-else class="divide-y divide-default">
        <div
          v-for="config in configStore.configs"
          :key="config.id"
          class="flex items-center justify-between px-4 py-3 hover:bg-muted"
        >
          <div class="flex items-center gap-3">
            <div>
              <div class="flex items-center gap-2">
                <span class="font-medium text-highlighted">{{ config.name }}</span>
                <UBadge v-if="config.is_default" color="success" variant="subtle" size="xs">
                  默认
                </UBadge>
              </div>
              <div class="text-sm text-muted">
                {{ config.model_name }} · {{ config.api_url || '默认端点' }}
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
    <div class="bg-elevated rounded-lg border border-default">
      <div class="px-4 py-3 border-b border-default flex items-center justify-between">
        <h2 class="text-lg font-medium text-highlighted">应用设置</h2>
        <UButton
          v-if="hasChanges"
          icon="i-lucide-rotate-ccw"
          color="neutral"
          variant="ghost"
          size="xs"
          @click="resetToDefault"
        >
          重置默认
        </UButton>
      </div>
      <div class="p-4 space-y-4">
        <!-- 并行处理数 -->
        <div class="flex items-center justify-between">
          <div>
            <div class="font-medium text-highlighted">并行处理数</div>
            <div class="text-sm text-muted">同时处理的文件数量（1-10）</div>
          </div>
          <div class="flex items-center gap-2">
            <UButton
              icon="i-lucide-minus"
              color="neutral"
              variant="ghost"
              size="xs"
              :disabled="globalSettings.settings.parallelCount <= 1"
              @click="globalSettings.updateSetting('parallelCount', globalSettings.settings.parallelCount - 1)"
            />
            <span class="w-8 text-center font-medium">{{ globalSettings.settings.parallelCount }}</span>
            <UButton
              icon="i-lucide-plus"
              color="neutral"
              variant="ghost"
              size="xs"
              :disabled="globalSettings.settings.parallelCount >= 10"
              @click="globalSettings.updateSetting('parallelCount', globalSettings.settings.parallelCount + 1)"
            />
          </div>
        </div>

        <USeparator />

        <!-- 自动保存 -->
        <div class="flex items-center justify-between">
          <div>
            <div class="font-medium text-highlighted">自动保存</div>
            <div class="text-sm text-muted">处理完成后自动保存结果</div>
          </div>
          <UToggle
            :model-value="globalSettings.settings.autoSave"
            @update:model-value="globalSettings.updateSetting('autoSave', $event)"
          />
        </div>

        <USeparator />

        <!-- 重复数据处理策略 -->
        <div class="flex items-center justify-between">
          <div>
            <div class="font-medium text-highlighted">重复数据处理</div>
            <div class="text-sm text-muted">当检测到重复记录时的处理方式</div>
          </div>
          <div class="flex items-center gap-2">
            <UButton
              v-for="option in duplicateOptions"
              :key="option.value"
              :color="globalSettings.settings.duplicateStrategy === option.value ? 'primary' : 'neutral'"
              :variant="globalSettings.settings.duplicateStrategy === option.value ? 'solid' : 'ghost'"
              size="xs"
              @click="globalSettings.updateSetting('duplicateStrategy', option.value)"
            >
              {{ option.label }}
            </UButton>
          </div>
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

          <UFormField label="模型 (Model)" name="model_name" required>
            <UInput v-model="configForm.model_name" placeholder="例如：gpt-4o, claude-3-sonnet..." class="w-full" />
          </UFormField>

          <UFormField label="API URL" name="api_url" required>
            <UInput v-model="configForm.api_url" placeholder="https://api.openai.com/v1" class="w-full" />
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
        <p class="text-default">
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
import { ref, reactive, computed, onMounted } from 'vue'
import { useConfigStore } from '~/stores/config'
import { useGlobalSettingsStore } from '~/stores/globalSettings'
import type { AiConfig } from '~/types'

definePageMeta({
  layout: 'default',
})

const toast = useToast()
const configStore = useConfigStore()
const globalSettings = useGlobalSettingsStore()

// 重复数据处理选项
const duplicateOptions = [
  { value: 'skip' as const, label: '跳过' },
  { value: 'update' as const, label: '更新' },
  { value: 'merge' as const, label: '合并' },
]

// 是否有修改
const hasChanges = computed(() => {
  return (
    globalSettings.settings.parallelCount !== 3 ||
    globalSettings.settings.autoSave !== true ||
    globalSettings.settings.duplicateStrategy !== 'skip'
  )
})

// 重置为默认
async function resetToDefault() {
  await globalSettings.resetSettings()
  toast.add({ title: '已重置为默认设置', color: 'success' })
}

// 配置表单
const showCreateModal = ref(false)
const editingConfig = ref<AiConfig | null>(null)
const saving = ref(false)
const configForm = reactive({
  name: '',
  model_name: '',
  api_key: '',
  api_url: '',
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
  configForm.model_name = config.model_name || ''
  configForm.api_key = ''
  configForm.api_url = config.api_url || ''
  configForm.is_default = config.is_default
  showCreateModal.value = true
}

// 关闭对话框
function closeModal() {
  showCreateModal.value = false
  editingConfig.value = null
  configForm.name = ''
  configForm.model_name = ''
  configForm.api_key = ''
  configForm.api_url = ''
  configForm.is_default = false
}

// 保存配置
async function saveConfig() {
  if (!configForm.name.trim() || !configForm.model_name.trim()) return

  saving.value = true
  try {
    if (editingConfig.value) {
      await configStore.updateConfig(editingConfig.value.id, {
        name: configForm.name.trim(),
        model_name: configForm.model_name.trim(),
        api_key: configForm.api_key.trim() || undefined,
        api_url: configForm.api_url.trim() || undefined,
        is_default: configForm.is_default,
      })
      toast.add({ title: '配置已更新', color: 'success' })
    } else {
      await configStore.createConfig({
        name: configForm.name.trim(),
        model_name: configForm.model_name.trim(),
        api_key: configForm.api_key.trim(),
        api_url: configForm.api_url.trim() || 'https://api.openai.com/v1',
        is_default: configForm.is_default,
      })
      toast.add({ title: '配置已创建', color: 'success' })
    }
    closeModal()
  } catch (error) {
    console.error('Failed to save config:', error)
    toast.add({ title: '保存失败', description: String(error), color: 'error' })
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
      toast.add({
        title: '连接成功',
        description: result.response ? `AI 响应: ${result.response}` : 'AI 配置连接测试通过',
        color: 'success',
      })
    } else {
      toast.add({
        title: '连接失败',
        description: result.message,
        color: 'error',
      })
    }
  } catch (error) {
    console.error('Failed to test config:', error)
    toast.add({
      title: '连接失败',
      description: '请检查网络连接和配置',
      color: 'error',
    })
  } finally {
    testingId.value = null
  }
}

// 设为默认
async function setDefaultConfig(id: number) {
  try {
    await configStore.setDefault(id)
    toast.add({ title: '已设为默认配置', color: 'success' })
  } catch (error) {
    console.error('Failed to set default config:', error)
    toast.add({ title: '设置失败', color: 'error' })
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
    toast.add({ title: '配置已删除', color: 'success' })
    showDeleteModal.value = false
    configToDelete.value = null
  } catch (error) {
    console.error('Failed to delete config:', error)
    toast.add({ title: '删除失败', color: 'error' })
  } finally {
    deleting.value = false
  }
}

onMounted(async () => {
  configStore.fetchConfigs()
  await globalSettings.loadSettings()
})
</script>
