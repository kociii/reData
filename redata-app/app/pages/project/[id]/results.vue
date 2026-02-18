<template>
  <div class="h-full flex flex-col -m-6">
    <!-- 工具栏 -->
    <div class="flex justify-between items-center px-6 py-3 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex-shrink-0">
      <div class="flex items-center gap-4">
        <span class="text-sm text-gray-500 dark:text-gray-400">
          共 {{ totalCount }} 条记录
        </span>
        <UInput
          v-model="searchQuery"
          icon="i-lucide-search"
          placeholder="搜索..."
          class="w-64"
        />
      </div>
      <div class="flex gap-2">
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

    <!-- 数据表格（表头始终显示） -->
    <div class="flex-1 overflow-auto">
      <table class="min-w-full">
        <thead class="bg-gray-50 dark:bg-gray-900 sticky top-0 z-10">
          <tr>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider w-16">
              #
            </th>
            <th
              v-for="field in fields"
              :key="field.id"
              class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider"
            >
              {{ field.field_label }}
            </th>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              来源
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200 dark:divide-gray-700 bg-white dark:bg-gray-800">
          <!-- 加载状态 -->
          <tr v-if="loading">
            <td :colspan="fields.length + 2" class="px-4 py-16 text-center">
              <UIcon name="i-lucide-refresh-cw" class="w-8 h-8 animate-spin text-primary mx-auto" />
            </td>
          </tr>
          <!-- 空状态 -->
          <tr v-else-if="records.length === 0">
            <td :colspan="fields.length + 2" class="px-4 py-16 text-center text-gray-500 dark:text-gray-400">
              导入文件并处理后将在此显示数据
            </td>
          </tr>
          <!-- 数据行 -->
          <tr
            v-for="(record, index) in records"
            :key="record.id"
            class="hover:bg-gray-50 dark:hover:bg-gray-900"
          >
            <td class="px-4 py-2.5 text-sm text-gray-500 dark:text-gray-400">
              {{ (currentPage - 1) * pageSize + index + 1 }}
            </td>
            <td
              v-for="field in fields"
              :key="field.id"
              class="px-4 py-2.5 text-sm text-gray-900 dark:text-white"
            >
              {{ record[field.field_name] || '-' }}
            </td>
            <td class="px-4 py-2.5 text-sm text-gray-500 dark:text-gray-400">
              <span class="truncate max-w-[150px] block" :title="record.source_file">
                {{ record.source_file }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 分页栏 -->
    <div v-if="totalCount > 0" class="flex justify-between items-center px-6 py-2.5 border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex-shrink-0">
      <span class="text-sm text-gray-500 dark:text-gray-400">
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useFieldStore } from '~/stores/field'
import { resultsApi } from '~/utils/api'

const route = useRoute()
const projectId = computed(() => Number(route.params.id))
const fieldStore = useFieldStore()

// 数据状态
const loading = ref(false)
const records = ref<Record<string, any>[]>([])
const totalCount = ref(0)
const currentPage = ref(1)
const pageSize = 50

// 搜索
const searchQuery = ref('')

// 字段定义
const fields = computed(() => fieldStore.fields)

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

// 加载数据
async function loadData() {
  loading.value = true
  try {
    const result = await resultsApi.query(projectId.value, {
      page: currentPage.value,
      page_size: pageSize,
      search: searchQuery.value || undefined,
      status: 'success',
    })
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
watch(searchQuery, () => {
  currentPage.value = 1
  loadData()
})

onMounted(async () => {
  await fieldStore.fetchFields(projectId.value)
  await loadData()
})
</script>