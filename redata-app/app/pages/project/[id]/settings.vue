<template>
  <div class="space-y-6">
    <!-- 基本信息 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <h3 class="text-base font-medium text-gray-900 dark:text-white">基本信息</h3>
      </div>
      <div class="p-4 space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <UFormField label="项目名称">
            <UInput v-model="editForm.name" />
          </UFormField>
          <UFormField label="创建时间">
            <UInput :model-value="formatDateTime(project?.created_at)" disabled />
          </UFormField>
        </div>
        <UFormField label="项目描述">
          <UTextarea v-model="editForm.description" :rows="2" />
        </UFormField>
        <div class="flex gap-2">
          <UButton :loading="saving" @click="saveBasicInfo">保存</UButton>
          <UButton color="neutral" variant="ghost" @click="resetForm">取消</UButton>
        </div>
      </div>
    </div>

    <!-- 字段定义摘要 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center">
        <h3 class="text-base font-medium text-gray-900 dark:text-white">字段定义</h3>
        <UButton size="xs" variant="soft" :to="`/project/${projectId}/fields`">
          编辑字段定义
        </UButton>
      </div>
      <div class="p-4">
        <div v-if="fieldStore.fieldCount === 0" class="text-sm text-gray-500 dark:text-gray-400">
          尚未定义字段，请先添加字段定义。
        </div>
        <ul v-else class="space-y-1.5">
          <li v-for="field in fieldStore.fields" :key="field.id" class="flex items-center gap-2 text-sm">
            <UBadge v-if="field.is_required" color="error" variant="subtle" size="xs">必填</UBadge>
            <span class="text-gray-900 dark:text-white">{{ field.field_label }}</span>
            <span class="text-gray-400">-</span>
            <span class="text-gray-500 dark:text-gray-400">{{ field.extraction_hint || field.field_type }}</span>
          </li>
        </ul>
        <p class="text-xs text-gray-400 mt-3">当前定义了 {{ fieldStore.fieldCount }} 个字段</p>
      </div>
    </div>

    <!-- 去重配置 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <h3 class="text-base font-medium text-gray-900 dark:text-white">去重配置</h3>
      </div>
      <div class="p-4 space-y-4">
        <USwitch v-model="dedupForm.enabled" label="启用去重" />
        <div v-if="dedupForm.enabled" class="space-y-3">
          <UFormField label="去重字段">
            <div class="flex flex-wrap gap-2">
              <UCheckbox
                v-for="field in fieldStore.fields"
                :key="field.id"
                :model-value="dedupForm.fields.includes(field.field_name)"
                :label="field.field_label"
                @update:model-value="toggleDedupField(field.field_name, $event)"
              />
            </div>
          </UFormField>
          <UFormField label="去重策略">
            <URadioGroup
              v-model="dedupForm.strategy"
              :items="[
                { value: 'skip', label: '跳过重复' },
                { value: 'update', label: '更新已存在' },
                { value: 'merge', label: '保留标记' },
              ]"
            />
          </UFormField>
        </div>
        <UButton :loading="savingDedup" @click="saveDedupConfig">保存配置</UButton>
      </div>
    </div>

    <!-- 数据统计 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <h3 class="text-base font-medium text-gray-900 dark:text-white">数据统计</h3>
      </div>
      <div class="p-4 grid grid-cols-4 gap-4">
        <div>
          <div class="text-sm text-gray-500 dark:text-gray-400">总记录数</div>
          <div class="text-xl font-bold text-gray-900 dark:text-white">0</div>
        </div>
        <div>
          <div class="text-sm text-gray-500 dark:text-gray-400">今日新增</div>
          <div class="text-xl font-bold text-gray-900 dark:text-white">0</div>
        </div>
        <div>
          <div class="text-sm text-gray-500 dark:text-gray-400">本周新增</div>
          <div class="text-xl font-bold text-gray-900 dark:text-white">0</div>
        </div>
        <div>
          <div class="text-sm text-gray-500 dark:text-gray-400">最后处理</div>
          <div class="text-xl font-bold text-gray-900 dark:text-white">-</div>
        </div>
      </div>
    </div>

    <!-- 危险操作 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-red-200 dark:border-red-900">
      <div class="px-4 py-3 border-b border-red-200 dark:border-red-900">
        <h3 class="text-base font-medium text-red-600 dark:text-red-400">危险操作</h3>
      </div>
      <div class="p-4 flex gap-3">
        <UButton color="neutral" variant="soft" @click="exportConfig">导出项目配置</UButton>
        <UButton color="warning" variant="soft" @click="clearData">清空所有数据</UButton>
        <UButton color="error" variant="soft" @click="deleteProject">删除项目</UButton>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProjectStore } from '~/stores/project'
import { useFieldStore } from '~/stores/field'

const route = useRoute()
const router = useRouter()
const projectStore = useProjectStore()
const fieldStore = useFieldStore()

const projectId = computed(() => Number(route.params.id))
const project = computed(() => projectStore.currentProject)

const saving = ref(false)
const savingDedup = ref(false)

const editForm = reactive({ name: '', description: '' })
const dedupForm = reactive({ enabled: true, strategy: 'skip', fields: [] as string[] })

onMounted(() => {
  if (project.value) {
    editForm.name = project.value.name
    editForm.description = project.value.description || ''
    dedupForm.enabled = project.value.dedup_enabled
    dedupForm.strategy = project.value.dedup_strategy || 'skip'
  }
})

function resetForm() {
  if (project.value) {
    editForm.name = project.value.name
    editForm.description = project.value.description || ''
  }
}

async function saveBasicInfo() {
  saving.value = true
  try {
    await projectStore.updateProject(projectId.value, {
      name: editForm.name.trim(),
      description: editForm.description.trim() || undefined,
    })
  } catch (error) {
    console.error('Failed to save:', error)
  } finally {
    saving.value = false
  }
}

function toggleDedupField(fieldName: string, checked: boolean) {
  if (checked) {
    dedupForm.fields.push(fieldName)
  } else {
    dedupForm.fields = dedupForm.fields.filter(f => f !== fieldName)
  }
}

async function saveDedupConfig() {
  savingDedup.value = true
  try {
    await projectStore.updateProject(projectId.value, {
      dedup_enabled: dedupForm.enabled,
      dedup_strategy: dedupForm.strategy,
    })
  } catch (error) {
    console.error('Failed to save dedup config:', error)
  } finally {
    savingDedup.value = false
  }
}

function exportConfig() {
  console.log('Export config for project:', projectId.value)
}

function clearData() {
  if (confirm('确定要清空所有数据吗？此操作无法撤销。')) {
    console.log('Clear data for project:', projectId.value)
  }
}

async function deleteProject() {
  if (confirm('确定要删除此项目吗？所有数据将被永久删除。')) {
    try {
      await projectStore.deleteProject(projectId.value)
      router.push('/')
    } catch (error) {
      console.error('Failed to delete project:', error)
    }
  }
}

function formatDateTime(dateStr?: string): string {
  if (!dateStr) return '-'
  return new Date(dateStr).toLocaleString('zh-CN')
}
</script>
