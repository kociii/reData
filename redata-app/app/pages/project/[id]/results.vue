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
          批次：{{ batchFilter }}
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
        <!-- 数据来源按钮 -->
        <UButton
          icon="i-lucide-folder-tree"
          color="neutral"
          variant="ghost"
          @click="showSourcePanel = !showSourcePanel"
        >
          数据来源
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
          @click="exportData"
        >
          导出
        </UButton>
      </div>
    </div>

    <!-- 主内容区 -->
    <div class="flex-1 flex min-h-0">
      <!-- 数据来源面板 -->
      <div
        v-if="showSourcePanel"
        class="w-80 border-r border-default bg-elevated flex-shrink-0 overflow-y-auto"
      >
        <div class="p-4">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-medium">数据来源</h3>
            <UButton
              icon="i-lucide-refresh-cw"
              color="neutral"
              variant="ghost"
              size="xs"
              :loading="loadingBatches"
              @click="loadBatches"
            />
          </div>

          <!-- 加载状态 -->
          <div v-if="loadingBatches" class="py-8 text-center text-muted">
            <UIcon name="i-lucide-refresh-cw" class="w-6 h-6 animate-spin mx-auto mb-2" />
            <span class="text-sm">加载中...</span>
          </div>

          <!-- 空状态 -->
          <div v-else-if="batches.length === 0" class="py-8 text-center text-muted">
            <UIcon name="i-lucide-folder-open" class="w-8 h-8 mx-auto mb-2 opacity-50" />
            <span class="text-sm">暂无导入数据</span>
          </div>

          <!-- 批次列表 -->
          <div v-else class="space-y-3">
            <div
              v-for="batch in batches"
              :key="batch.batch_number"
              class="border border-default rounded-lg overflow-hidden"
            >
              <!-- 批次头部 -->
              <div
                class="flex items-center justify-between px-3 py-2 bg-muted cursor-pointer hover:bg-default"
                @click="toggleBatch(batch.batch_number)"
              >
                <div class="flex items-center gap-2 min-w-0">
                  <UIcon
                    :name="expandedBatches.has(batch.batch_number) ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
                    class="w-4 h-4 flex-shrink-0 text-muted"
                  />
                  <span class="text-sm font-medium truncate">{{ batch.batch_number }}</span>
                  <UBadge
                    v-if="batch.status === 'rolled_back'"
                    color="warning"
                    variant="subtle"
                    size="xs"
                  >
                    已撤回
                  </UBadge>
                </div>
                <div class="flex items-center gap-2 flex-shrink-0">
                  <span class="text-xs text-muted">{{ batch.total_records }} 条</span>
                  <UButton
                    v-if="batch.total_records > 0"
                    icon="i-lucide-undo-2"
                    color="error"
                    variant="ghost"
                    size="xs"
                    @click.stop="confirmRollback('batch', batch)"
                  />
                </div>
              </div>

              <!-- 批次内容（文件列表） -->
              <div v-if="expandedBatches.has(batch.batch_number)" class="border-t border-default">
                <div
                  v-for="file in batch.files"
                  :key="file.file_name"
                  class="border-b border-default last:border-b-0"
                >
                  <!-- 文件头部 -->
                  <div
                    class="flex items-center justify-between px-3 py-2 pl-8 cursor-pointer hover:bg-muted"
                    @click="toggleFile(batch.batch_number, file.file_name)"
                  >
                    <div class="flex items-center gap-2 min-w-0">
                      <UIcon
                        :name="expandedFiles.has(`${batch.batch_number}:${file.file_name}`) ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
                        class="w-4 h-4 flex-shrink-0 text-muted"
                      />
                      <UIcon name="i-lucide-file-spreadsheet" class="w-4 h-4 flex-shrink-0 text-muted" />
                      <span class="text-sm truncate">{{ file.file_name }}</span>
                    </div>
                    <div class="flex items-center gap-2 flex-shrink-0">
                      <span class="text-xs text-muted">{{ file.total_records }} 条</span>
                      <UButton
                        v-if="file.total_records > 0"
                        icon="i-lucide-undo-2"
                        color="error"
                        variant="ghost"
                        size="xs"
                        @click.stop="confirmRollback('file', batch, file)"
                      />
                    </div>
                  </div>

                  <!-- Sheet 列表 -->
                  <div v-if="expandedFiles.has(`${batch.batch_number}:${file.file_name}`)" class="border-t border-default">
                    <div
                      v-for="sheet in file.sheets"
                      :key="sheet.sheet_name"
                      class="flex items-center justify-between px-3 py-2 pl-14 hover:bg-muted"
                    >
                      <div class="flex items-center gap-2 min-w-0">
                        <UIcon name="i-lucide-table" class="w-4 h-4 flex-shrink-0 text-muted" />
                        <span class="text-sm truncate">{{ sheet.sheet_name }}</span>
                      </div>
                      <div class="flex items-center gap-2 flex-shrink-0">
                        <span class="text-xs text-muted">{{ sheet.record_count }} 条</span>
                        <UButton
                          v-if="sheet.record_count > 0"
                          icon="i-lucide-undo-2"
                          color="error"
                          variant="ghost"
                          size="xs"
                          @click="confirmRollback('sheet', batch, file, sheet)"
                        />
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

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
      <span class="text-sm text-muted">
        显示 {{ (currentPage - 1) * pageSize + 1 }} - {{ Math.min(currentPage * pageSize, totalCount) }} / {{ totalCount }} 条
      </span>
      <UPagination
        v-model:page="currentPage"
        :total="totalCount"
        :items-per-page="pageSize"
        show-edges
        :sibling-count="1"
        size="sm"
      />
    </div>

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
import { useFieldStore } from '~/stores/field'
import { resultsApi, batchesApi } from '~/utils/api'
import type { BatchDetailResponse, FileImportDetail, SheetImportDetail } from '~/types'

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
const pageSize = 50

// 搜索（带防抖）
const searchQuery = ref('')
let searchTimeout: ReturnType<typeof setTimeout> | null = null

// 字段定义
const fields = computed(() => fieldStore.fields)

// 原始数据弹窗
const rawDataModalOpen = ref(false)
const rawDataModalContent = ref('')

function openRawDataModal(rawData: string) {
  rawDataModalContent.value = rawData
  rawDataModalOpen.value = true
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

// 导出
const exportData = async () => {
  try {
    const blob = await resultsApi.export(projectId.value, 'xlsx')
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `results_${projectId.value}.xlsx`
    a.click()
    URL.revokeObjectURL(url)
  } catch (error) {
    console.error('Export failed:', error)
  }
}

// 数据来源面板
const showSourcePanel = ref(false)
const loadingBatches = ref(false)
const batches = ref<BatchDetailResponse[]>([])
const expandedBatches = ref<Set<string>>(new Set())
const expandedFiles = ref<Set<string>>(new Set())

async function loadBatches() {
  loadingBatches.value = true
  try {
    batches.value = await batchesApi.listWithStats(projectId.value)
  } catch (error: any) {
    console.error('Failed to load batches:', error)
    toast.add({
      title: '加载批次失败',
      description: error?.message || String(error),
      color: 'error',
    })
  } finally {
    loadingBatches.value = false
  }
}

function toggleBatch(batchNumber: string) {
  const newSet = new Set(expandedBatches.value)
  if (newSet.has(batchNumber)) {
    newSet.delete(batchNumber)
  } else {
    newSet.add(batchNumber)
  }
  expandedBatches.value = newSet
}

function toggleFile(batchNumber: string, fileName: string) {
  const key = `${batchNumber}:${fileName}`
  const newSet = new Set(expandedFiles.value)
  if (newSet.has(key)) {
    newSet.delete(key)
  } else {
    newSet.add(key)
  }
  expandedFiles.value = newSet
}

// 撤回功能
const rollbackModalOpen = ref(false)
const rollingBack = ref(false)
const rollbackTarget = ref<{
  type: 'batch' | 'file' | 'sheet'
  batchNumber: string
  fileName?: string
  sheetName?: string
  description: string
  count: number
} | null>(null)

function confirmRollback(
  type: 'batch' | 'file' | 'sheet',
  batch: BatchDetailResponse,
  file?: FileImportDetail,
  sheet?: SheetImportDetail
) {
  let description = ''
  let count = 0

  if (type === 'batch') {
    description = `批次 "${batch.batch_number}"`
    count = batch.total_records
  } else if (type === 'file' && file) {
    description = `文件 "${file.file_name}"`
    count = file.total_records
  } else if (type === 'sheet' && file && sheet) {
    description = `Sheet "${sheet.sheet_name}"`
    count = sheet.record_count
  }

  rollbackTarget.value = {
    type,
    batchNumber: batch.batch_number,
    fileName: file?.file_name,
    sheetName: sheet?.sheet_name,
    description,
    count,
  }
  rollbackModalOpen.value = true
}

async function executeRollback() {
  if (!rollbackTarget.value) return

  rollingBack.value = true
  try {
    let result
    const target = rollbackTarget.value

    if (target.type === 'batch') {
      result = await batchesApi.rollback(projectId.value, target.batchNumber)
    } else if (target.type === 'file' && target.fileName) {
      result = await batchesApi.rollbackFile(projectId.value, target.batchNumber, target.fileName)
    } else if (target.type === 'sheet' && target.fileName && target.sheetName) {
      result = await batchesApi.rollbackSheet(projectId.value, target.batchNumber, target.fileName, target.sheetName)
    }

    toast.add({
      title: '撤回成功',
      description: result?.message,
      color: 'success',
    })

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
  loading.value = true
  try {
    const result = await resultsApi.query(projectId.value, {
      page: currentPage.value,
      page_size: pageSize,
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
  await Promise.all([loadData(), loadBatches()])
})

onUnmounted(() => {
  if (searchTimeout) clearTimeout(searchTimeout)
})
</script>
