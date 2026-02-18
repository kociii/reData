<template>
  <div class="py-4">
    <!-- 工具栏 -->
    <div class="flex justify-between items-center mb-4">
      <div class="text-sm text-gray-500 dark:text-gray-400">
        共 {{ fieldStore.fieldCount }} 个字段
      </div>
      <UButton icon="i-lucide-plus" @click="openFieldModal()">
        添加字段
      </UButton>
    </div>

    <!-- 空状态 -->
    <div
      v-if="!fieldStore.loading && fieldStore.fieldCount === 0"
      class="text-center py-12 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700"
    >
      <UIcon name="i-lucide-table-2" class="w-12 h-12 mx-auto text-gray-400 mb-4" />
      <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">还没有字段定义</h3>
      <p class="text-gray-500 dark:text-gray-400 mb-6">添加字段定义后，系统将根据这些字段提取数据</p>
      <UButton icon="i-lucide-plus" @click="openFieldModal()">
        添加字段
      </UButton>
    </div>

    <!-- 字段表格 -->
    <div v-else class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
        <thead class="bg-gray-50 dark:bg-gray-900">
          <tr>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider w-12">
              必填
            </th>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider w-12">
              去重
            </th>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              字段名称
            </th>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              字段类型
            </th>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              字段英文名
            </th>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              验证规则
            </th>
            <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider w-24">
              操作
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
          <tr
            v-for="field in fieldStore.fields"
            :key="field.id"
            class="hover:bg-gray-50 dark:hover:bg-gray-900"
          >
            <td class="px-4 py-3 text-center">
              <UIcon v-if="field.is_required" name="i-lucide-check" class="w-4 h-4 text-green-600" />
            </td>
            <td class="px-4 py-3 text-center">
              <UIcon v-if="field.is_dedup_key" name="i-lucide-fingerprint" class="w-4 h-4 text-primary-600" />
            </td>
            <td class="px-4 py-3 text-sm text-gray-900 dark:text-white">
              {{ field.field_label }}
            </td>
            <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">
              {{ getFieldTypeLabel(field.field_type) }}
            </td>
            <td class="px-4 py-3 text-sm font-mono text-gray-600 dark:text-gray-400">
              {{ field.field_name }}
            </td>
            <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">
              {{ field.validation_rule || '-' }}
            </td>
            <td class="px-4 py-3 text-right">
              <div class="flex justify-end gap-1">
                <UButton
                  icon="i-lucide-pencil"
                  color="neutral"
                  variant="ghost"
                  size="xs"
                  @click="openFieldModal(field)"
                />
                <UButton
                  icon="i-lucide-trash-2"
                  color="error"
                  variant="ghost"
                  size="xs"
                  @click="confirmDelete(field)"
                />
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 添加/编辑字段对话框 -->
    <UModal v-model:open="showFieldModal" :title="editingField ? '编辑字段' : '添加字段'">
      <template #body>
        <div class="space-y-5 py-2">
          <UFormField label="字段名称" name="field_label" required>
            <UInput v-model="fieldForm.field_label" placeholder="例如：客户姓名" class="w-full" />
          </UFormField>

          <UFormField label="字段类型" name="field_type" required>
            <USelect v-model="fieldForm.field_type" :items="fieldTypes" class="w-full" />
          </UFormField>

          <USwitch v-model="fieldForm.is_required" label="必填字段" />

          <USwitch v-model="fieldForm.is_dedup_key" label="参与去重" />

          <div v-if="editingField" class="pt-4 border-t border-gray-200 dark:border-gray-700 space-y-3">
            <div class="text-sm font-medium text-gray-700 dark:text-gray-300">AI 生成的字段信息</div>
            <div class="space-y-2 text-sm">
              <div class="flex justify-between">
                <span class="text-gray-500 dark:text-gray-400">字段名:</span>
                <span class="font-mono text-gray-900 dark:text-white">{{ editingField.field_name }}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-500 dark:text-gray-400">验证规则:</span>
                <span class="text-gray-900 dark:text-white">{{ editingField.validation_rule || '无' }}</span>
              </div>
            </div>
          </div>
        </div>
      </template>

      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" @click="closeFieldModal">
            取消
          </UButton>
          <UButton :loading="saving" @click="saveField">
            保存
          </UButton>
        </div>
      </template>
    </UModal>

    <!-- 删除确认对话框 -->
    <UModal v-model:open="showDeleteModal" title="确认删除">
      <template #body>
        <p class="text-gray-600 dark:text-gray-400">
          确定要删除字段 "{{ fieldToDelete?.field_label }}" 吗？此操作无法撤销。
        </p>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" @click="showDeleteModal = false">
            取消
          </UButton>
          <UButton color="error" :loading="deleting" @click="deleteField">
            删除
          </UButton>
        </div>
      </template>
    </UModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted } from 'vue'
import { useFieldStore } from '~/stores/field'
import { useConfigStore } from '~/stores/config'
import { fieldsApi, aiServiceApi } from '~/utils/api'
import type { ProjectField } from '~/types'

const route = useRoute()
const toast = useToast()
const projectId = computed(() => Number(route.params.id))
const fieldStore = useFieldStore()
const configStore = useConfigStore()

// 字段类型选项
const fieldTypes = [
  { value: 'text', label: '文本' },
  { value: 'number', label: '数字' },
  { value: 'phone', label: '手机号' },
  { value: 'email', label: '邮箱' },
  { value: 'url', label: 'URL' },
  { value: 'date', label: '日期' },
  { value: 'address', label: '地址' },
  { value: 'company', label: '公司' },
]

function getFieldTypeLabel(type: string) {
  return fieldTypes.find(t => t.value === type)?.label || type
}

// 字段表单
const showFieldModal = ref(false)
const editingField = ref<ProjectField | null>(null)
const saving = ref(false)
const fieldForm = reactive({
  field_label: '',
  field_type: 'text',
  is_required: false,
  is_dedup_key: false,
})

function openFieldModal(field?: ProjectField) {
  if (field) {
    editingField.value = field
    fieldForm.field_label = field.field_label
    fieldForm.field_type = field.field_type
    fieldForm.is_required = field.is_required
    fieldForm.is_dedup_key = field.is_dedup_key || false
  } else {
    editingField.value = null
    fieldForm.field_label = ''
    fieldForm.field_type = 'text'
    fieldForm.is_required = false
    fieldForm.is_dedup_key = false
  }
  showFieldModal.value = true
}

function closeFieldModal() {
  showFieldModal.value = false
  editingField.value = null
}

async function saveField() {
  if (!fieldForm.field_label.trim()) return

  // 编辑模式下验证 ID 有效性
  if (editingField.value && (!editingField.value.id || editingField.value.id <= 0)) {
    toast.add({
      title: '无效的字段 ID',
      description: `字段 ID "${editingField.value.id}" 无效，请刷新页面后重试`,
      color: 'error',
    })
    console.error('[saveField] Invalid field ID:', editingField.value.id)
    return
  }

  saving.value = true
  try {
    // 编辑模式：检查字段标签是否变化
    const isLabelChanged = editingField.value &&
      editingField.value.field_label.trim() !== fieldForm.field_label.trim()

    let fieldName = editingField.value?.field_name || ''
    let validationRule = editingField.value?.validation_rule
    let extractionHint = editingField.value?.extraction_hint || ''

    console.log('[saveField] Starting save...', {
      isEditing: !!editingField.value,
      isLabelChanged,
      fieldName,
      validationRule,
      extractionHint
    })

    // 仅在新增或字段标签变化时调用 AI 翻译
    if (!editingField.value || isLabelChanged) {
      console.log('[saveField] Calling metadata generation...')
      // 尝试使用 AI 翻译字段名
      if (configStore.defaultConfig) {
        try {
          console.log('[saveField] Using AI translation with config:', configStore.defaultConfig.id)
          const aiResult = await aiServiceApi.generateFieldMetadataWithAI(
            configStore.defaultConfig.id,
            fieldForm.field_label,
            fieldForm.field_type
          )
          console.log('[saveField] AI result:', aiResult)
          fieldName = aiResult.field_name || generateFallbackFieldName(fieldForm.field_label)
          validationRule = aiResult.validation_rule || null
          extractionHint = aiResult.extraction_hint || `提取${fieldForm.field_label.trim()}字段`
        } catch (aiError) {
          console.warn('[saveField] AI 翻译失败，使用本地生成:', aiError)
          // AI 失败时使用本地生成
          const localResult = await fieldsApi.generateMetadata({
            field_label: fieldForm.field_label,
            field_type: fieldForm.field_type,
          })
          fieldName = localResult.field_name || generateFallbackFieldName(fieldForm.field_label)
          validationRule = localResult.validation_rule || null
          extractionHint = localResult.extraction_hint || `提取${fieldForm.field_label.trim()}字段`
        }
      } else {
        console.log('[saveField] No AI config, using local generation')
        // 没有 AI 配置时使用本地生成
        const localResult = await fieldsApi.generateMetadata({
          field_label: fieldForm.field_label,
          field_type: fieldForm.field_type,
        })
        fieldName = localResult.field_name || generateFallbackFieldName(fieldForm.field_label)
        validationRule = localResult.validation_rule || null
        extractionHint = localResult.extraction_hint || `提取${fieldForm.field_label.trim()}字段`
      }
    }

    // 确保 fieldName 不为空
    if (!fieldName || !fieldName.trim()) {
      fieldName = generateFallbackFieldName(fieldForm.field_label)
    }

    console.log('[saveField] Final values:', {
      fieldName,
      validationRule,
      extractionHint,
      projectId: projectId.value
    })

    if (editingField.value) {
      console.log('[saveField] Calling updateField with id:', editingField.value.id)
      await fieldStore.updateField(editingField.value.id, {
        field_label: fieldForm.field_label.trim(),
        field_type: fieldForm.field_type,
        is_required: fieldForm.is_required,
        is_dedup_key: fieldForm.is_dedup_key,
        field_name: fieldName,
        validation_rule: validationRule,
        extraction_hint: extractionHint,
      })
      toast.add({ title: '字段已更新', color: 'success' })
    } else {
      console.log('[saveField] Calling createField with project_id:', projectId.value)
      await fieldStore.createField({
        project_id: projectId.value,
        field_label: fieldForm.field_label.trim(),
        field_type: fieldForm.field_type,
        is_required: fieldForm.is_required,
        is_dedup_key: fieldForm.is_dedup_key,
        field_name: fieldName,
        validation_rule: validationRule,
        extraction_hint: extractionHint,
      })
      toast.add({ title: '字段已创建', color: 'success' })
    }

    closeFieldModal()
  } catch (error: any) {
    console.error('Failed to save field:', error)
    toast.add({
      title: '保存失败',
      description: error?.message || String(error),
      color: 'error',
    })
  } finally {
    saving.value = false
  }
}

// 删除
const showDeleteModal = ref(false)
const fieldToDelete = ref<ProjectField | null>(null)
const deleting = ref(false)

function confirmDelete(field: ProjectField) {
  fieldToDelete.value = field
  showDeleteModal.value = true
}

async function deleteField() {
  if (!fieldToDelete.value) return

  deleting.value = true
  try {
    console.log('[deleteField] Deleting field id:', fieldToDelete.value.id)
    await fieldStore.deleteField(fieldToDelete.value.id)
    showDeleteModal.value = false
    fieldToDelete.value = null
    toast.add({ title: '字段已删除', color: 'success' })
  } catch (error: any) {
    console.error('Failed to delete field:', error)
    toast.add({
      title: '删除失败',
      description: error?.message || String(error),
      color: 'error',
    })
  } finally {
    deleting.value = false
  }
}

// 生成本地备用字段名
function generateFallbackFieldName(label: string): string {
  const mappings: Record<string, string> = {
    '姓名': 'name', '名字': 'name', '电话': 'phone', '手机': 'phone',
    '手机号': 'phone', '邮箱': 'email', '地址': 'address', '公司': 'company',
    '日期': 'date', '金额': 'amount', '价格': 'price', '数量': 'quantity',
    '备注': 'remark', '标题': 'title', '编号': 'id', '状态': 'status',
  }

  const trimmed = label.trim()
  if (mappings[trimmed]) {
    return mappings[trimmed]
  }

  for (const [cn, en] of Object.entries(mappings)) {
    if (trimmed.includes(cn)) {
      return en
    }
  }

  return trimmed.toLowerCase().replace(/\s+/g, '_').replace(/[^a-z0-9_]/g, '') || 'field'
}

// 初始化
onMounted(async () => {
  await fieldStore.fetchFields(projectId.value)
  await configStore.fetchConfigs()
})
</script>