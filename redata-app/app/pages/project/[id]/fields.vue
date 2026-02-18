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
              字段名
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
import { ref, computed, reactive } from 'vue'
import { useFieldStore } from '~/stores/field'
import { fieldsApi } from '~/utils/api'
import type { ProjectField } from '~/types'

const route = useRoute()
const toast = useToast()
const projectId = computed(() => Number(route.params.id))
const fieldStore = useFieldStore()

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

  saving.value = true
  try {
    // 编辑模式：检查字段名称是否变化
    const isLabelChanged = editingField.value &&
      editingField.value.field_label.trim() !== fieldForm.field_label.trim()

    let fieldName = editingField.value?.field_name || ''
    let validationRule = editingField.value?.validation_rule || null
    let extractionHint = editingField.value?.extraction_hint || ''

    // 仅在新增或字段名称变化时调用 AI
    if (!editingField.value || isLabelChanged) {
      const aiResult = await fieldsApi.generateMetadata({
        field_label: fieldForm.field_label,
        field_type: fieldForm.field_type,
      })
      fieldName = aiResult.field_name
      validationRule = aiResult.validation_rule
      extractionHint = aiResult.extraction_hint
    }

    if (editingField.value) {
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
    await fieldStore.deleteField(fieldToDelete.value.id)
    showDeleteModal.value = false
    fieldToDelete.value = null
  } catch (error) {
    console.error('Failed to delete field:', error)
  } finally {
    deleting.value = false
  }
}
</script>