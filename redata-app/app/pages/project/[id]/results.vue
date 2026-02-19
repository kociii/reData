<template>
  <div class="h-full flex flex-col">
    <!-- 工具栏 -->
    <div class="flex justify-between items-center px-6 py-3 border-b border-default bg-elevated flex-shrink-0">
      <div class="flex items-center gap-4">
        <span class="text-sm text-muted">
          共 {{ totalCount }} 条记录
        </span>
        <!-- 批次过滤提示 -->
        <UBadge
          v-if="batchFilter"
          color="primary"
          variant="subtle"
          class="flex items-center gap-1 cursor-pointer"
          @click="clearBatchFilter"
        >
          导入文件：{{ batches.find(b => b.batch_number === batchFilter)?.source_file ?? batchFilter }}
          <UIcon name="i-lucide-x" class="w-3 h-3" />
        </UBadge>
        <UInput
          v-model="searchQuery"
          icon="i-lucide-search"
          placeholder="搜索..."
          class="w-64"
        />
      </div>
      <div class="flex gap-2">
        <!-- 筛选按钮 -->
        <UButton
          icon="i-lucide-filter"
          color="neutral"
          variant="ghost"
          :class="{ 'bg-primary/10 text-primary': showFilterPanel || hasActiveFilters }"
          @click="showFilterPanel = !showFilterPanel"
        >
          筛选
          <UBadge
            v-if="activeFilterCount > 0"
            color="primary"
            size="xs"
            class="ml-1"
          >
            {{ activeFilterCount }}
          </UBadge>
        </UButton>
        <UButton
          v-if="selectedIds.size > 0"
          icon="i-lucide-trash-2"
          color="error"
          variant="ghost"
          @click="confirmBatchDelete"
        >
          删除选中 ({{ selectedIds.size }})
        </UButton>
        <UButton
          icon="i-lucide-download"
          color="neutral"
          variant="ghost"
          @click="openExportModal"
        >
          导出
        </UButton>
      </div>
    </div>

    <!-- 筛选面板 -->
    <div v-if="showFilterPanel" class="border-b border-default bg-elevated flex-shrink-0">
      <div class="px-6 py-3 space-y-3">
        <!-- 快捷筛选 -->
        <div class="flex items-center gap-4 flex-wrap">
          <div class="flex items-center gap-2">
            <span class="text-xs text-muted">来源文件：</span>
            <USelectMenu
              v-model="filterSourceFile"
              :items="sourceFileOptions"
              placeholder="全部"
              size="xs"
              class="w-48"
              value-key="value"
              @update:model-value="applyFilters"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="text-xs text-muted">导入文件：</span>
            <USelectMenu
              v-model="filterBatch"
              :items="batchOptions"
              placeholder="全部"
              size="xs"
              class="w-48"
              value-key="value"
              @update:model-value="applyFilters"
            />
            <!-- 当选中批次时显示撤回按钮 -->
            <UButton
              v-if="filterBatch"
              icon="i-lucide-undo-2"
              color="error"
              variant="ghost"
              size="xs"
              @click="confirmRollbackBatch(filterBatch)"
            >
              撤回此批次
            </UButton>
          </div>
        </div>

        <!-- 筛选条件 -->
        <div v-if="filterConditions.length > 0" class="space-y-2">
          <div
            v-for="(condition, index) in filterConditions"
            :key="condition.id"
            class="flex items-center gap-2"
          >
            <!-- 字段选择 -->
            <USelectMenu
              v-model="condition.field"
              :items="fieldOptions"
              placeholder="选择字段"
              size="xs"
              class="w-32"
              value-key="value"
              @update:model-value="onConditionFieldChange(condition)"
            />
            <!-- 运算符选择 -->
            <USelectMenu
              v-model="condition.operator"
              :items="getOperatorOptions(condition.field)"
              placeholder="运算符"
              size="xs"
              class="w-28"
              value-key="value"
            />
            <!-- 值输入 -->
            <template v-if="condition.operator !== 'is_empty' && condition.operator !== 'is_not_empty'">
              <UInput
                v-if="condition.operator !== 'between'"
                v-model="condition.value as string"
                placeholder="值"
                size="xs"
                class="w-40"
              />
              <div v-else class="flex items-center gap-1">
                <UInput
                  v-model="(condition.value as [string, string])[0]"
                  placeholder="起始"
                  size="xs"
                  class="w-24"
                />
                <span class="text-xs text-muted">-</span>
                <UInput
                  v-model="(condition.value as [string, string])[1]"
                  placeholder="结束"
                  size="xs"
                  class="w-24"
                />
              </div>
            </template>
            <!-- 删除按钮 -->
            <UButton
              icon="i-lucide-x"
              color="neutral"
              variant="ghost"
              size="xs"
              @click="removeCondition(index)"
            />
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="flex items-center gap-2">
          <UButton
            icon="i-lucide-plus"
            color="neutral"
            variant="ghost"
            size="xs"
            @click="addCondition"
          >
            添加条件
          </UButton>
          <div class="flex items-center gap-2 ml-4">
            <span class="text-xs text-muted">条件组合：</span>
            <URadioGroup
              v-model="filterConjunction"
              :items="[{ value: 'and', label: '且' }, { value: 'or', label: '或' }]"
              size="xs"
              @change="applyFilters"
            />
          </div>
          <div class="flex-1" />
          <UButton
            color="neutral"
            variant="ghost"
            size="xs"
            :disabled="!hasActiveFilters"
            @click="clearAllFilters"
          >
            重置筛选
          </UButton>
          <UButton
            color="primary"
            size="xs"
            :disabled="filterConditions.length === 0 && !filterSourceFile && !filterBatch"
            @click="applyFilters"
          >
            应用筛选
          </UButton>
        </div>
      </div>
    </div>

    <!-- 主内容区 -->
    <div class="flex-1 flex min-h-0">
      <!-- 数据表格（支持横向滚动） -->
      <div class="flex-1 overflow-auto">
        <table class="min-w-max">
          <thead class="bg-muted sticky top-0 z-10">
            <tr>
              <th class="px-4 py-3 text-left text-xs font-medium text-muted uppercase tracking-wider w-10">
                <UCheckbox
                  :model-value="isAllSelected"
                  :indeterminate="isPartialSelected"
                  @update:model-value="toggleSelectAll"
                />
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-muted uppercase tracking-wider w-16">
                #
              </th>
              <th
                v-for="field in fields"
                :key="field.id"
                class="px-4 py-3 text-left text-xs font-medium text-muted uppercase tracking-wider whitespace-nowrap"
                style="min-width: 100px; max-width: 180px;"
              >
                {{ field.field_label }}
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-muted uppercase tracking-wider whitespace-nowrap" style="min-width: 130px;">
                导入时间
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-muted uppercase tracking-wider whitespace-nowrap" style="min-width: 120px;">
                来源
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-muted uppercase tracking-wider whitespace-nowrap" style="min-width: 200px;">
                原始数据
              </th>
              <th class="px-4 py-3 text-center text-xs font-medium text-muted uppercase tracking-wider w-20">
                操作
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-default bg-elevated">
            <!-- 加载状态 -->
            <tr v-if="loading">
              <td :colspan="fields.length + 5" class="px-4 py-16 text-center">
                <UIcon name="i-lucide-refresh-cw" class="w-8 h-8 animate-spin text-primary mx-auto" />
              </td>
            </tr>
            <!-- 空状态 -->
            <tr v-else-if="records.length === 0">
              <td :colspan="fields.length + 5" class="px-4 py-16 text-center text-muted">
                导入文件并处理后将在此显示数据
              </td>
            </tr>
            <!-- 数据行 -->
            <tr
              v-for="(record, index) in records"
              :key="record.id"
              class="hover:bg-muted"
              :class="{ 'bg-error/5': selectedIds.has(record.id) }"
            >
              <td class="px-4 py-2.5">
                <UCheckbox
                  :model-value="selectedIds.has(record.id)"
                  @update:model-value="toggleSelect(record.id)"
                />
              </td>
              <td class="px-4 py-2.5 text-sm text-muted">
                {{ (currentPage - 1) * pageSize + index + 1 }}
              </td>
              <td
                v-for="field in fields"
                :key="field.id"
                class="px-4 py-2.5 text-sm text-highlighted"
                style="max-width: 180px;"
              >
                <div class="whitespace-pre-wrap break-words">
                  {{ record[field.id] || '-' }}
                </div>
              </td>
              <td class="px-4 py-2.5 text-sm text-muted whitespace-nowrap">
                {{ formatImportTime(record.created_at) }}
              </td>
              <td class="px-4 py-2.5 text-sm text-muted" style="max-width: 150px;">
                <div class="whitespace-pre-wrap break-words">
                  {{ record.source_file || '-' }}
                  <span v-if="record.source_sheet" class="text-xs text-dimmed block">
                    / {{ record.source_sheet }}
                  </span>
                </div>
              </td>
              <td class="px-4 py-2.5 text-sm text-default" style="max-width: 300px;">
                <div v-if="record.raw_data" class="flex items-start gap-2">
                  <div
                    class="font-mono text-xs bg-muted rounded p-1.5 flex-1 overflow-hidden"
                    style="display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; word-break: break-all;"
                  >
                    {{ record.raw_data }}
                  </div>
                  <UButton
                    size="xs"
                    color="neutral"
                    variant="ghost"
                    @click="openRawDataModal(record.raw_data)"
                  >
                    查看
                  </UButton>
                </div>
                <span v-else class="text-dimmed">-</span>
              </td>
              <td class="px-4 py-2.5 text-center">
                <UButton
                  icon="i-lucide-trash-2"
                  color="error"
                  variant="ghost"
                  size="xs"
                  @click="confirmDelete(record)"
                />
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- 分页栏 -->
    <div v-if="totalCount > 0" class="flex justify-between items-center px-6 py-2.5 border-t border-default bg-elevated flex-shrink-0">
      <div class="flex items-center gap-3">
        <span class="text-sm text-muted">
          显示 {{ (currentPage - 1) * pageSize + 1 }} - {{ Math.min(currentPage * pageSize, totalCount) }} / {{ totalCount }} 条
        </span>
        <USelectMenu
          v-model="pageSize"
          :items="pageSizeOptions"
          value-key="value"
          size="xs"
          class="w-28"
        />
      </div>
      <UPagination
        v-model:page="currentPage"
        :total="totalCount"
        :items-per-page="pageSize"
        show-edges
        :sibling-count="1"
        size="sm"
      />
    </div>

    <!-- 导出对话框 -->
    <UModal v-model:open="exportModalOpen" title="导出数据">
      <template #body>
        <div class="space-y-5">
          <!-- 字段选择 -->
          <div>
            <div class="flex items-center justify-between mb-2">
              <span class="text-sm font-medium">选择导出字段</span>
              <div class="flex gap-2">
                <button class="text-xs text-primary hover:underline" @click="exportSelectAll">全选</button>
                <span class="text-xs text-dimmed">·</span>
                <button class="text-xs text-muted hover:underline" @click="exportSelectNone">清空</button>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-1.5 max-h-48 overflow-y-auto pr-1">
              <label
                v-for="field in fields"
                :key="field.id"
                class="flex items-center gap-2 text-sm cursor-pointer select-none px-2 py-1.5 rounded hover:bg-muted"
              >
                <UCheckbox
                  :model-value="exportFieldIds.includes(String(field.id))"
                  @update:model-value="toggleExportField(String(field.id), $event)"
                />
                <span class="truncate">{{ field.field_label }}</span>
              </label>
              <!-- 固定附加列 -->
              <label class="flex items-center gap-2 text-sm cursor-pointer select-none px-2 py-1.5 rounded hover:bg-muted">
                <UCheckbox v-model="exportIncludeImportTime" />
                <span class="truncate text-muted">导入时间</span>
              </label>
              <label class="flex items-center gap-2 text-sm cursor-pointer select-none px-2 py-1.5 rounded hover:bg-muted">
                <UCheckbox v-model="exportIncludeSourceFile" />
                <span class="truncate text-muted">来源文件</span>
              </label>
            </div>
          </div>

          <!-- 导出范围 -->
          <div>
            <span class="text-sm font-medium block mb-2">导出范围</span>
            <div class="space-y-2">
              <label class="flex items-center gap-2 cursor-pointer">
                <input v-model="exportRange" type="radio" value="all" />
                <span class="text-sm">全部数据</span>
                <span class="text-xs text-muted">所有成功记录（不受当前筛选影响）</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input v-model="exportRange" type="radio" value="filtered" />
                <span class="text-sm">当前筛选结果</span>
                <span class="text-xs text-muted">（共 {{ totalCount }} 条）</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input v-model="exportRange" type="radio" value="current_page" />
                <span class="text-sm">仅当前页</span>
                <span class="text-xs text-muted">（{{ records.length }} 条）</span>
              </label>
            </div>
          </div>
        </div>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" @click="exportModalOpen = false">取消</UButton>
          <UButton
            icon="i-lucide-download"
            :loading="exportLoading"
            :disabled="exportFieldIds.length === 0 && !exportIncludeImportTime && !exportIncludeSourceFile"
            @click="executeExport"
          >
            导出 {{ exportRangeLabel }}
          </UButton>
        </div>
      </template>
    </UModal>

    <!-- 原始数据详情弹窗 -->
    <UModal v-model:open="rawDataModalOpen">
      <template #content>
        <div class="p-4">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-base font-medium">原始数据</h3>
            <UButton
              icon="i-lucide-x"
              color="neutral"
              variant="ghost"
              size="sm"
              @click="rawDataModalOpen = false"
            />
          </div>
          <div
            class="font-mono text-xs bg-muted rounded p-3 max-h-96 overflow-auto whitespace-pre-wrap break-words"
          >
            {{ rawDataModalContent }}
          </div>
        </div>
      </template>
    </UModal>

    <!-- 删除确认弹窗 -->
    <UModal v-model:open="deleteModalOpen" title="确认删除">
      <template #body>
        <p class="text-default">
          确定要删除 {{ recordToDelete ? '这条记录' : `${selectedIds.size} 条记录` }}吗？此操作无法撤销。
        </p>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" @click="deleteModalOpen = false">
            取消
          </UButton>
          <UButton color="error" :loading="deleting" @click="executeDelete">
            删除
          </UButton>
        </div>
      </template>
    </UModal>

    <!-- 撤回确认弹窗 -->
    <UModal v-model:open="rollbackModalOpen" title="确认撤回">
      <template #body>
        <div class="space-y-3">
          <p class="text-default">
            确定要撤回 {{ rollbackTarget?.description }}吗？
          </p>
          <p class="text-sm text-muted">
            将删除 {{ rollbackTarget?.count }} 条记录，此操作不可恢复。
          </p>
          <div class="bg-warning/10 border border-warning/20 rounded-lg p-3">
            <p class="text-sm text-warning">
              提示：撤回后可以在本地处理数据，然后重新导入。
            </p>
          </div>
        </div>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" @click="rollbackModalOpen = false">
            取消
          </UButton>
          <UButton color="error" :loading="rollingBack" @click="executeRollback">
            确认撤回
          </UButton>
        </div>
      </template>
    </UModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { save } from '@tauri-apps/plugin-dialog'
import { useFieldStore } from '~/stores/field'
import { resultsApi, batchesApi } from '~/utils/api'
import type { BatchDetailResponse, FilterCondition, SourceFileInfo, AdvancedFilterRequest } from '~/types'
import { OPERATOR_LABELS, getOperatorsForFieldType } from '~/types'

const route = useRoute()
const router = useRouter()
const toast = useToast()
const projectId = computed(() => Number(route.params.id))
const fieldStore = useFieldStore()

// 批次过滤（从路由查询参数获取）
const batchFilter = computed(() => route.query.batch as string | undefined)

function clearBatchFilter() {
  router.push(`/project/${projectId.value}/results`)
}

// 数据状态
const loading = ref(false)
const records = ref<Record<string, any>[]>([])
const totalCount = ref(0)
const currentPage = ref(1)
const pageSize = ref(50)
const pageSizeOptions = [
  { label: '50 条/页', value: 50 },
  { label: '100 条/页', value: 100 },
  { label: '200 条/页', value: 200 },
  { label: '500 条/页', value: 500 },
]

// 搜索（带防抖）
const searchQuery = ref('')
let searchTimeout: ReturnType<typeof setTimeout> | null = null

// 字段定义
const fields = computed(() => fieldStore.fields)

// ============ 筛选相关 ============

const showFilterPanel = ref(false)
const filterConditions = ref<FilterCondition[]>([])
const filterConjunction = ref<'and' | 'or'>('and')
const filterSourceFile = ref<string | null>(null)
const filterBatch = ref<string | null>(null)

// 来源文件选项
const sourceFiles = ref<SourceFileInfo[]>([])
const sourceFileOptions = computed(() => [
  { label: '全部', value: null },
  ...sourceFiles.value.map(f => ({ label: `${f.source_file} (${f.record_count})`, value: f.source_file }))
])

// 导入记录选项（显示文件名而非批次号）
const batchOptions = computed(() => [
  { label: '全部', value: null },
  ...batches.value.map(b => ({
    label: `${b.source_file} (${b.total_records}条)`,
    value: b.batch_number
  }))
])

// 字段选项
const fieldOptions = computed(() =>
  fields.value.map(f => ({
    label: f.field_label,
    value: String(f.id),
    fieldType: f.field_type
  }))
)

// 获取运算符选项
function getOperatorOptions(fieldId: string | null) {
  if (!fieldId) return []
  const field = fields.value.find(f => String(f.id) === fieldId)
  if (!field) return []
  const operators = getOperatorsForFieldType(field.field_type)
  return operators.map(op => ({
    label: OPERATOR_LABELS[op],
    value: op
  }))
}

// 条件字段变更时重置运算符
function onConditionFieldChange(condition: FilterCondition) {
  condition.operator = 'eq'
  condition.value = ''
}

// 添加筛选条件
function addCondition() {
  const firstField = fields.value[0]
  filterConditions.value.push({
    id: `cond-${Date.now()}`,
    field: firstField ? String(firstField.id) : '',
    operator: 'eq',
    value: ''
  })
}

// 移除筛选条件
function removeCondition(index: number) {
  filterConditions.value.splice(index, 1)
}

// 活跃筛选条件数量
const activeFilterCount = computed(() => {
  let count = filterConditions.value.length
  if (filterSourceFile.value) count++
  if (filterBatch.value) count++
  return count
})

// 是否有活跃的筛选条件
const hasActiveFilters = computed(() =>
  filterConditions.value.length > 0 ||
  filterSourceFile.value !== null ||
  filterBatch.value !== null
)

// 清除所有筛选
function clearAllFilters() {
  filterConditions.value = []
  filterSourceFile.value = null
  filterBatch.value = null
  filterConjunction.value = 'and'
  applyFilters()
}

// 应用筛选
async function applyFilters() {
  currentPage.value = 1
  await loadDataAdvanced()
}

// 加载来源文件列表
async function loadSourceFiles() {
  try {
    sourceFiles.value = await resultsApi.getSourceFiles(projectId.value)
  } catch (error) {
    console.error('Failed to load source files:', error)
  }
}

// 高级筛选查询
async function loadDataAdvanced() {
  loading.value = true
  try {
    // 构建筛选请求
    const validConditions = filterConditions.value.filter(c => c.field && c.operator)

    const filterRequest = {
      search: searchQuery.value || undefined,
      conditions: validConditions.map(c => ({
        field: c.field,
        operator: c.operator,
        value: c.value
      })),
      source_file: filterSourceFile.value || undefined,
      batch_number: filterBatch.value || batchFilter.value || undefined,
      conjunction: filterConjunction.value
    }

    const result = await resultsApi.queryAdvanced(
      projectId.value,
      filterRequest,
      currentPage.value,
      pageSize.value
    )
    records.value = result.records
    totalCount.value = result.total
  } catch (error) {
    console.error('Failed to load data:', error)
    records.value = []
    totalCount.value = 0
  } finally {
    loading.value = false
  }
}

// 原始数据弹窗
const rawDataModalOpen = ref(false)
const rawDataModalContent = ref('')

function openRawDataModal(rawData: string) {
  rawDataModalContent.value = rawData
  rawDataModalOpen.value = true
}

function formatImportTime(isoString: string | undefined): string {
  if (!isoString) return '-'
  try {
    const d = new Date(isoString)
    if (isNaN(d.getTime())) return '-'
    const year = d.getFullYear()
    const month = String(d.getMonth() + 1).padStart(2, '0')
    const day = String(d.getDate()).padStart(2, '0')
    const hour = String(d.getHours()).padStart(2, '0')
    const min = String(d.getMinutes()).padStart(2, '0')
    return `${year}-${month}-${day} ${hour}:${min}`
  }
  catch {
    return '-'
  }
}

// 选择状态
const selectedIds = ref<Set<number>>(new Set())

// 全选状态
const isAllSelected = computed(() => {
  return records.value.length > 0 && records.value.every(r => selectedIds.value.has(r.id))
})

const isPartialSelected = computed(() => {
  const selectedCount = records.value.filter(r => selectedIds.value.has(r.id)).length
  return selectedCount > 0 && selectedCount < records.value.length
})

function toggleSelect(id: number) {
  const newSet = new Set(selectedIds.value)
  if (newSet.has(id)) {
    newSet.delete(id)
  } else {
    newSet.add(id)
  }
  selectedIds.value = newSet
}

function toggleSelectAll() {
  if (isAllSelected.value) {
    // 取消选择当前页所有
    const newSet = new Set(selectedIds.value)
    records.value.forEach(r => newSet.delete(r.id))
    selectedIds.value = newSet
  } else {
    // 选择当前页所有
    const newSet = new Set(selectedIds.value)
    records.value.forEach(r => newSet.add(r.id))
    selectedIds.value = newSet
  }
}

// 删除功能
const deleteModalOpen = ref(false)
const recordToDelete = ref<Record<string, any> | null>(null)
const deleting = ref(false)

function confirmDelete(record: Record<string, any>) {
  recordToDelete.value = record
  deleteModalOpen.value = true
}

function confirmBatchDelete() {
  recordToDelete.value = null
  deleteModalOpen.value = true
}

async function executeDelete() {
  deleting.value = true
  try {
    if (recordToDelete.value) {
      // 删除单条
      await resultsApi.delete(projectId.value, recordToDelete.value.id)
      toast.add({ title: '记录已删除', color: 'success' })
    } else {
      // 批量删除
      const ids = Array.from(selectedIds.value)
      for (const id of ids) {
        await resultsApi.delete(projectId.value, id)
      }
      toast.add({ title: `已删除 ${ids.length} 条记录`, color: 'success' })
      selectedIds.value = new Set()
    }
    deleteModalOpen.value = false
    recordToDelete.value = null
    await loadData()
    await loadBatches()
  } catch (error: any) {
    toast.add({
      title: '删除失败',
      description: error?.message || String(error),
      color: 'error',
    })
  } finally {
    deleting.value = false
  }
}

// ============ 导出 ============

const exportModalOpen = ref(false)
const exportLoading = ref(false)
const exportFieldIds = ref<string[]>([])
const exportIncludeImportTime = ref(true)
const exportIncludeSourceFile = ref(true)
const exportRange = ref<'all' | 'filtered' | 'current_page'>('all')

// 导出按钮标签
const exportRangeLabel = computed(() => {
  if (exportRange.value === 'current_page') return `${records.value.length} 条`
  if (exportRange.value === 'filtered') return `${totalCount.value} 条`
  return '所有数据'
})

function openExportModal() {
  exportFieldIds.value = fields.value.map(f => String(f.id))
  exportIncludeImportTime.value = true
  exportIncludeSourceFile.value = true
  exportRange.value = hasActiveFilters.value ? 'filtered' : 'all'
  exportModalOpen.value = true
}

function toggleExportField(fieldId: string, checked: boolean) {
  if (checked) {
    if (!exportFieldIds.value.includes(fieldId)) exportFieldIds.value = [...exportFieldIds.value, fieldId]
  }
  else {
    exportFieldIds.value = exportFieldIds.value.filter(id => id !== fieldId)
  }
}

function exportSelectAll() {
  exportFieldIds.value = fields.value.map(f => String(f.id))
  exportIncludeImportTime.value = true
  exportIncludeSourceFile.value = true
}

function exportSelectNone() {
  exportFieldIds.value = []
  exportIncludeImportTime.value = false
  exportIncludeSourceFile.value = false
}

// 构建当前筛选请求（用于导出）
function buildCurrentFilter(): AdvancedFilterRequest {
  const validConditions = filterConditions.value.filter(c => c.field && c.operator)
  return {
    search: searchQuery.value || undefined,
    conditions: validConditions,
    source_file: filterSourceFile.value || undefined,
    batch_number: filterBatch.value || batchFilter.value || undefined,
    conjunction: filterConjunction.value,
  }
}

async function executeExport() {
  // 打开原生保存对话框
  const savePath = await save({
    defaultPath: `export_${new Date().toISOString().slice(0, 10)}.xlsx`,
    filters: [{ name: 'Excel 工作簿', extensions: ['xlsx'] }],
  })
  if (!savePath) return  // 用户取消

  exportLoading.value = true
  try {
    const selectedFields = fields.value.filter(f => exportFieldIds.value.includes(String(f.id)))
    const fieldIds = selectedFields.map(f => String(f.id))
    const fieldLabels = selectedFields.map(f => f.field_label)

    let filter: AdvancedFilterRequest | null = null
    let exportPage: number | null = null
    let exportPageSize: number | null = null

    if (exportRange.value === 'filtered') {
      filter = buildCurrentFilter()
    } else if (exportRange.value === 'current_page') {
      filter = buildCurrentFilter()
      exportPage = currentPage.value
      exportPageSize = pageSize.value
    }
    // 'all': filter = null，导出全部成功记录

    const count = await resultsApi.exportXlsx(projectId.value, {
      filePath: savePath,
      fieldIds,
      fieldLabels,
      includeImportTime: exportIncludeImportTime.value,
      includeSourceFile: exportIncludeSourceFile.value,
      filter,
      page: exportPage,
      pageSize: exportPageSize,
    })

    exportModalOpen.value = false
    toast.add({ title: `已导出 ${count} 条记录`, color: 'success' })
  }
  catch (error: any) {
    toast.add({ title: '导出失败', description: error?.message || String(error), color: 'error' })
  }
  finally {
    exportLoading.value = false
  }
}

// 数据来源批次列表（用于批次筛选器选项）
const batches = ref<BatchDetailResponse[]>([])

async function loadBatches() {
  try {
    batches.value = await batchesApi.listWithStats(projectId.value)
  } catch (error: any) {
    console.error('Failed to load batches:', error)
  }
}

// 撤回功能
const rollbackModalOpen = ref(false)
const rollingBack = ref(false)
const rollbackTarget = ref<{
  batchNumber: string
  description: string
  count: number
} | null>(null)

// 通过批次号触发撤回
function confirmRollbackBatch(batchNumber: string | null) {
  if (!batchNumber) return
  const batch = batches.value.find(b => b.batch_number === batchNumber)
  rollbackTarget.value = {
    batchNumber,
    description: `文件 "${batch?.source_file ?? batchNumber}"`,
    count: batch?.total_records ?? 0,
  }
  rollbackModalOpen.value = true
}

async function executeRollback() {
  if (!rollbackTarget.value) return

  rollingBack.value = true
  try {
    const result = await batchesApi.rollback(projectId.value, rollbackTarget.value.batchNumber)

    toast.add({
      title: '撤回成功',
      description: result?.message,
      color: 'success',
    })

    // 如果撤回的是当前筛选的批次，清除批次筛选
    if (filterBatch.value === rollbackTarget.value.batchNumber) {
      filterBatch.value = null
    }

    rollbackModalOpen.value = false
    rollbackTarget.value = null

    // 刷新数据
    await Promise.all([loadData(), loadBatches()])
  } catch (error: any) {
    toast.add({
      title: '撤回失败',
      description: error?.message || String(error),
      color: 'error',
    })
  } finally {
    rollingBack.value = false
  }
}

// 加载数据
async function loadData() {
  // 如果有筛选条件，使用高级查询
  if (hasActiveFilters.value) {
    await loadDataAdvanced()
    return
  }

  loading.value = true
  try {
    const result = await resultsApi.query(projectId.value, {
      page: currentPage.value,
      page_size: pageSize.value,
      search: searchQuery.value || undefined,
      status: 'success',
      batch_number: batchFilter.value,
    })
    // raw_data 为索引格式字符串：1:列1内容;2:列2内容;...n:列n内容;
    records.value = result.records
    totalCount.value = result.total
  } catch (error) {
    console.error('Failed to load data:', error)
    records.value = []
    totalCount.value = 0
  } finally {
    loading.value = false
  }
}

watch(currentPage, () => loadData())
watch(pageSize, () => {
  currentPage.value = 1
  loadData()
})
watch(batchFilter, () => {
  currentPage.value = 1
  loadData()
})
watch(searchQuery, () => {
  // 防抖：延迟 300ms 后执行搜索
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    currentPage.value = 1
    loadData()
  }, 300)
})

onMounted(async () => {
  await fieldStore.fetchFields(projectId.value)
  await Promise.all([loadData(), loadBatches(), loadSourceFiles()])
})

onUnmounted(() => {
  if (searchTimeout) clearTimeout(searchTimeout)
})
</script>
