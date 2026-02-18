<template>
  <div class="flex flex-col">
    <!-- 工具栏 -->
    <div class="flex justify-between items-center mb-4">
      <div class="text-sm text-muted">
        <template v-if="store.hasActiveTasks">
          {{ store.activeTasks.length }} 个任务进行中
        </template>
        <template v-else>
          暂无进行中的任务
        </template>
      </div>
      <div class="flex gap-2">
        <UButton
          icon="i-lucide-upload"
          :loading="selectingFiles"
          @click="selectFiles"
        >
          导入文件
        </UButton>
        <UButton
          v-if="store.hasPendingFiles"
          icon="i-lucide-play"
          color="success"
          :loading="startingAll"
          @click="startAllPending"
        >
          全部开始 ({{ store.pendingFiles.length }})
        </UButton>
      </div>
    </div>

    <!-- 空状态 -->
    <div
      v-if="store.tasks.length === 0 && !store.hasPendingFiles"
      class="text-center py-16 bg-elevated rounded-lg border border-default"
    >
      <UIcon name="i-lucide-file-up" class="w-12 h-12 mx-auto text-dimmed mb-4" />
      <h3 class="text-lg font-medium text-highlighted mb-2">还没有处理任务</h3>
      <p class="text-muted mb-6">导入 Excel 文件开始数据处理</p>
      <UButton icon="i-lucide-upload" :loading="selectingFiles" @click="selectFiles">
        导入文件
      </UButton>
    </div>

    <!-- 主内容区：左右分栏 -->
    <div v-else class="flex gap-4" style="height: calc(100vh - 280px);">
      <!-- 左侧面板：三状态分组 -->
      <div class="w-72 flex-shrink-0 flex flex-col gap-3 overflow-y-auto">
        <!-- 待处理文件分组 -->
        <div v-if="store.hasPendingFiles" class="bg-elevated rounded-lg border border-default">
          <div
            class="flex justify-between items-center p-2 cursor-pointer"
            @click="store.toggleGroupCollapse('pending')"
          >
            <div class="flex items-center gap-2">
              <UIcon
                :name="store.isGroupCollapsed('pending') ? 'i-lucide-chevron-right' : 'i-lucide-chevron-down'"
                class="w-4 h-4 text-dimmed"
              />
              <h3 class="text-xs font-medium text-muted uppercase tracking-wider">
                待处理
              </h3>
              <UBadge color="neutral" variant="subtle" size="xs">{{ store.pendingFiles.length }}</UBadge>
            </div>
            <UButton
              variant="ghost" size="xs" color="neutral" icon="i-lucide-trash-2"
              @click.stop="store.clearPendingFiles"
            />
          </div>
          <div v-if="!store.isGroupCollapsed('pending')" class="px-2 pb-2 space-y-1">
            <div
              v-for="file in store.pendingFiles" :key="file.id"
              class="flex items-center gap-2 p-2 rounded-lg bg-muted text-sm"
            >
              <UIcon name="i-lucide-file-spreadsheet" class="w-4 h-4 text-success flex-shrink-0" />
              <span class="truncate flex-1 text-highlighted text-xs">{{ file.name }}</span>
              <UButton
                size="xs" variant="ghost" color="neutral" icon="i-lucide-x"
                @click="store.removePendingFile(file.id)"
              />
            </div>
          </div>
        </div>

        <!-- 处理中分组 -->
        <div v-if="store.processingTasks.length > 0" class="bg-elevated rounded-lg border border-default">
          <div
            class="flex justify-between items-center p-2 cursor-pointer"
            @click="store.toggleGroupCollapse('processing')"
          >
            <div class="flex items-center gap-2">
              <UIcon
                :name="store.isGroupCollapsed('processing') ? 'i-lucide-chevron-right' : 'i-lucide-chevron-down'"
                class="w-4 h-4 text-dimmed"
              />
              <h3 class="text-xs font-medium text-primary uppercase tracking-wider">
                处理中
              </h3>
              <UBadge color="primary" variant="subtle" size="xs">{{ store.processingTasks.length }}</UBadge>
            </div>
            <!-- 批量操作菜单 -->
            <UDropdownMenu
              v-if="selectedProcessingIds.length > 0"
              :items="batchActionsForProcessing"
            >
              <UButton
                variant="ghost" size="xs" color="neutral"
                icon="i-lucide-more-horizontal"
                @click.stop
              />
            </UDropdownMenu>
          </div>
          <div v-if="!store.isGroupCollapsed('processing')" class="px-2 pb-2 space-y-1">
            <div
              v-for="task in store.processingTasks" :key="task.id"
              class="flex items-center gap-2 p-2 rounded-lg border text-sm cursor-pointer transition-colors"
              :class="[
                task.id === store.selectedTaskId
                  ? 'bg-primary/10 border-primary/30'
                  : 'bg-muted border-default hover:bg-accented',
                store.isSelected(task.id) ? 'ring-2 ring-primary' : ''
              ]"
              @click="handleTaskClick(task.id, $event)"
            >
              <UCheckbox
                :model-value="store.isSelected(task.id)"
                @update:model-value="store.toggleTaskSelection(task.id)"
                @click.stop
              />
              <UIcon :name="getStatusIcon(task.status)" :class="getStatusIconClass(task.status)" class="flex-shrink-0" />
              <div class="min-w-0 flex-1">
                <div class="font-medium text-highlighted truncate text-xs">
                  {{ getTaskTitle(task) }}
                </div>
                <div class="text-xs text-muted">
                  {{ getTaskSubtitle(task) }} · {{ getStatusText(task.status) }}
                </div>
              </div>
              <UBadge :color="getStatusColor(task.status)" variant="subtle" size="xs">
                {{ task.success_count }}
              </UBadge>
            </div>
          </div>
        </div>

        <!-- 已完成分组 -->
        <div v-if="store.completedTasks.length > 0" class="bg-elevated rounded-lg border border-default">
          <div
            class="flex justify-between items-center p-2 cursor-pointer"
            @click="store.toggleGroupCollapse('completed')"
          >
            <div class="flex items-center gap-2">
              <UIcon
                :name="store.isGroupCollapsed('completed') ? 'i-lucide-chevron-right' : 'i-lucide-chevron-down'"
                class="w-4 h-4 text-dimmed"
              />
              <h3 class="text-xs font-medium text-success uppercase tracking-wider">
                已完成
              </h3>
              <UBadge color="success" variant="subtle" size="xs">{{ store.completedTasks.length }}</UBadge>
            </div>
          </div>
          <div v-if="!store.isGroupCollapsed('completed')" class="px-2 pb-2 space-y-1">
            <div
              v-for="task in store.completedTasks" :key="task.id"
              class="flex items-center gap-2 p-2 rounded-lg border text-sm cursor-pointer transition-colors"
              :class="task.id === store.selectedTaskId
                ? 'bg-primary/10 border-primary/30'
                : 'bg-muted border-default hover:bg-accented'"
              @click="handleTaskClick(task.id, $event)"
            >
              <UIcon :name="getStatusIcon(task.status)" :class="getStatusIconClass(task.status)" class="flex-shrink-0" />
              <div class="min-w-0 flex-1">
                <div class="font-medium text-highlighted truncate text-xs">
                  {{ getTaskTitle(task) }}
                </div>
                <div class="text-xs text-muted">
                  {{ getTaskSubtitle(task) }} · {{ getStatusText(task.status) }}
                </div>
              </div>
              <UBadge :color="getStatusColor(task.status)" variant="subtle" size="xs">
                {{ task.success_count }}
              </UBadge>
            </div>
          </div>
        </div>
      </div>

      <!-- 右侧面板：对话式展示 -->
      <div class="flex-1 flex flex-col gap-4 min-w-0 overflow-y-auto">
        <template v-if="store.selectedTask">
          <!-- 步骤条 -->
          <UStepper
            :items="stepperItems"
            :default-value="currentStepIndex"
            disabled
            size="sm"
            class="w-full"
          />

          <!-- 统计数据：左右分布 -->
          <div class="flex justify-between items-center px-4 py-3 bg-elevated rounded-lg border border-default">
            <div class="text-center">
              <div class="text-2xl font-bold text-highlighted">{{ store.selectedTask.total_rows || 0 }}</div>
              <div class="text-xs text-muted mt-1">原始数据</div>
            </div>
            <div class="h-8 w-px bg-default" />
            <div class="text-center">
              <div class="text-2xl font-bold text-success">{{ store.selectedTask.success_count }}</div>
              <div class="text-xs text-muted mt-1">导入成功</div>
            </div>
          </div>

          <!-- 任务控制按钮（处理中时显示） -->
          <div
            v-if="store.selectedTask.status === 'processing' || store.selectedTask.status === 'paused'"
            class="flex gap-2 justify-end"
          >
            <UButton
              v-if="store.selectedTask.status === 'processing'"
              icon="i-lucide-pause" color="neutral" variant="ghost" size="xs"
              @click="pauseTask(store.selectedTask.id)"
            >
              暂停
            </UButton>
            <UButton
              v-if="store.selectedTask.status === 'paused'"
              icon="i-lucide-play" color="neutral" variant="ghost" size="xs"
              @click="resumeTask(store.selectedTask.id)"
            >
              继续
            </UButton>
            <UButton
              icon="i-lucide-square" color="error" variant="ghost" size="xs"
              @click="cancelTask(store.selectedTask.id)"
            >
              取消
            </UButton>
          </div>

          <!-- 视图切换按钮 -->
          <div class="flex gap-1 p-1 bg-muted rounded-lg">
            <UButton
              variant="ghost"
              size="xs"
              :color="detailViewMode === 'logs' ? 'primary' : 'neutral'"
              :class="detailViewMode === 'logs' ? 'bg-elevated' : ''"
              icon="i-lucide-scroll-text"
              @click="switchViewMode('logs')"
            >
              处理过程
            </UButton>
            <UButton
              variant="ghost"
              size="xs"
              :color="detailViewMode === 'raw' ? 'primary' : 'neutral'"
              :class="detailViewMode === 'raw' ? 'bg-elevated' : ''"
              icon="i-lucide-file-spreadsheet"
              @click="switchViewMode('raw')"
            >
              原始数据
            </UButton>
            <UButton
              variant="ghost"
              size="xs"
              :color="detailViewMode === 'results' ? 'primary' : 'neutral'"
              :class="detailViewMode === 'results' ? 'bg-elevated' : ''"
              icon="i-lucide-table-2"
              @click="switchViewMode('results')"
            >
              处理结果
            </UButton>
          </div>

          <!-- 处理过程（日志）视图 -->
          <div v-if="detailViewMode === 'logs'" class="flex-1 min-h-0 flex flex-col">
            <div class="flex justify-between items-center mb-2">
              <h3 class="text-xs font-medium text-muted uppercase tracking-wider">处理日志</h3>
              <UButton
                variant="ghost" size="xs" color="neutral" icon="i-lucide-trash-2"
                @click="clearSelectedTaskLogs"
              />
            </div>
            <div
              ref="logContainer"
              class="flex-1 overflow-y-auto space-y-2 min-h-40 max-h-64 bg-muted rounded-lg p-3"
            >
              <div v-if="store.selectedLogs.length === 0" class="text-dimmed text-center py-8">
                等待日志...
              </div>
              <div
                v-for="(log, index) in store.selectedLogs" :key="index"
                class="flex"
                :class="log.align === 'right' ? 'justify-end' : 'justify-start'"
              >
                <div
                  class="max-w-[85%] px-3 py-2 rounded-lg text-sm"
                  :class="getLogBubbleClass(log)"
                >
                  <!-- AI 日志特殊样式 -->
                  <template v-if="log.category === 'ai'">
                    <div class="flex items-center gap-2 mb-1">
                      <UIcon name="i-lucide-sparkles" class="w-3 h-3 text-purple-500" />
                      <span class="text-xs text-purple-500 font-medium">AI 分析</span>
                    </div>
                  </template>
                  <div class="flex items-start gap-2">
                    <span class="text-xs opacity-60 flex-shrink-0">{{ log.time }}</span>
                    <span>{{ log.message }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 原始数据视图 -->
          <div v-else-if="detailViewMode === 'raw'" class="flex-1 min-h-0 flex flex-col">
            <div v-if="rawDataLoading" class="flex-1 flex items-center justify-center">
              <UIcon name="i-lucide-loader" class="w-6 h-6 animate-spin text-primary" />
            </div>
            <div v-else-if="!rawData" class="flex-1 flex flex-col items-center justify-center text-dimmed gap-3">
              <UIcon name="i-lucide-file-spreadsheet" class="w-10 h-10" />
              <p class="text-sm">点击选择文件查看原始数据</p>
              <UButton size="sm" icon="i-lucide-folder-open" @click="loadRawData">
                选择文件
              </UButton>
            </div>
            <template v-else>
              <!-- 数据来源信息 -->
              <div v-if="rawSheets.length > 0" class="text-xs text-muted mb-2">
                来源: {{ rawSheets.map(s => s.name).join(', ') }}
              </div>
              <!-- 数据表格 -->
              <div class="flex-1 overflow-auto bg-muted rounded-lg">
                <table class="w-full text-xs border-collapse">
                  <thead class="sticky top-0 bg-elevated">
                    <tr>
                      <th class="border-b border-default px-2 py-1 text-left text-muted font-medium w-8">#</th>
                      <th
                        v-for="(header, colIndex) in (rawData.rows[0] || [])"
                        :key="colIndex"
                        class="border-b border-default px-2 py-1 text-left text-muted font-medium"
                      >
                        字段{{ header }}
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="(row, rowIndex) in rawData.rows" :key="rowIndex" class="hover:bg-elevated/50">
                      <td class="border-b border-default px-2 py-1 text-muted">{{ rowIndex + 1 }}</td>
                      <td
                        v-for="(cell, colIndex) in row"
                        :key="colIndex"
                        class="border-b border-default px-2 py-1 truncate max-w-[120px]"
                        :title="String(cell)"
                      >
                        {{ cell ?? '' }}
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </template>
          </div>

          <!-- 处理结果视图 -->
          <div v-else-if="detailViewMode === 'results'" class="flex-1 min-h-0 flex flex-col">
            <div v-if="resultsLoading" class="flex-1 flex items-center justify-center">
              <UIcon name="i-lucide-loader" class="w-6 h-6 animate-spin text-primary" />
            </div>
            <div v-else-if="resultsData.length === 0" class="flex-1 flex flex-col items-center justify-center text-dimmed gap-2">
              <UIcon name="i-lucide-inbox" class="w-10 h-10" />
              <p class="text-sm">暂无处理结果</p>
              <p class="text-xs text-muted">任务完成后会显示在这里</p>
            </div>
            <template v-else>
              <div class="flex justify-between items-center mb-2">
                <span class="text-xs text-muted">共 {{ resultsTotal }} 条记录</span>
              </div>
              <!-- 结果表格 -->
              <div class="flex-1 overflow-auto bg-muted rounded-lg">
                <table class="w-full text-xs border-collapse">
                  <thead class="sticky top-0 bg-elevated">
                    <tr>
                      <th class="border-b border-default px-2 py-1 text-left text-muted font-medium w-8">#</th>
                      <th class="border-b border-default px-2 py-1 text-left text-muted font-medium">来源</th>
                      <th class="border-b border-default px-2 py-1 text-left text-muted font-medium">数据</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="(record, index) in resultsData" :key="record.id" class="hover:bg-elevated/50">
                      <td class="border-b border-default px-2 py-1 text-muted">{{ index + 1 }}</td>
                      <td class="border-b border-default px-2 py-1 text-muted whitespace-nowrap">
                        <div>{{ record.source_file }}</div>
                        <div v-if="record.source_sheet" class="text-dimmed">Sheet: {{ record.source_sheet }}</div>
                      </td>
                      <td class="border-b border-default px-2 py-1">
                        <div class="flex flex-wrap gap-1">
                          <template v-for="(value, key) in record.data" :key="key">
                            <span v-if="value" class="inline-flex items-center gap-1 bg-elevated px-1.5 py-0.5 rounded text-default">
                              <span class="text-muted">{{ key }}:</span>
                              <span class="truncate max-w-[100px]" :title="String(value)">{{ value }}</span>
                            </span>
                          </template>
                        </div>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </template>
          </div>
        </template>

        <!-- 未选中任务时的提示 -->
        <div v-else class="flex-1 flex items-center justify-center text-dimmed">
          <div class="text-center">
            <UIcon name="i-lucide-mouse-pointer-click" class="w-10 h-10 mx-auto mb-3" />
            <p>选择左侧任务查看详情</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProcessingStore } from '~/stores/processing'
import { recordsApi } from '~/utils/api'
import type { PendingFile, ExcelPreview, ProjectRecord, ProjectField } from '~/types'

const route = useRoute()
const router = useRouter()
const toast = useToast()
const store = useProcessingStore()

const projectId = computed(() => Number(route.params.id))

const selectingFiles = ref(false)
const startingAll = ref(false)
const logContainer = ref<HTMLElement | null>(null)

// 右侧详情视图模式
const detailViewMode = ref<'logs' | 'raw' | 'results'>('logs')

// 原始数据状态
const rawDataLoading = ref(false)
const rawData = ref<ExcelPreview | null>(null)
const rawSheets = ref<{ name: string; row_count: number; column_count: number }[]>([])
const currentSheet = ref<string>('')

// 处理结果状态
const resultsLoading = ref(false)
const resultsData = ref<ProjectRecord[]>([])
const resultsTotal = ref(0)

// 项目字段（用于显示结果）
const projectFields = ref<ProjectField[]>([])

// 步骤条配置
const stepperItems = computed(() => {
  const stages = store.selectedStages
  return stages.map((stage, index) => ({
    title: stage.label,
    icon: stage.status === 'completed' ? 'i-lucide-check' :
          stage.status === 'active' ? 'i-lucide-loader' :
          stage.status === 'error' ? 'i-lucide-x' : 'i-lucide-circle',
    value: index,
  }))
})

// 当前步骤索引
const currentStepIndex = computed(() => {
  const stages = store.selectedStages
  const activeIndex = stages.findIndex(s => s.status === 'active')
  if (activeIndex >= 0) return activeIndex
  const completedCount = stages.filter(s => s.status === 'completed').length
  if (completedCount === stages.length) return stages.length - 1
  return 0
})

// 多选：已选中的处理中任务 ID
const selectedProcessingIds = computed(() => {
  return store.processingTasks
    .filter(t => store.isSelected(t.id))
    .map(t => t.id)
})

// 批量操作菜单项
const batchActionsForProcessing = computed(() => [[
  {
    label: '全部暂停',
    icon: 'i-lucide-pause',
    click: () => batchPause(),
    disabled: selectedProcessingIds.value.length === 0
  },
  {
    label: '全部恢复',
    icon: 'i-lucide-play',
    click: () => batchResume(),
    disabled: selectedProcessingIds.value.length === 0
  },
  {
    label: '全部取消',
    icon: 'i-lucide-square',
    click: () => batchCancel(),
    disabled: selectedProcessingIds.value.length === 0
  }
]])

// 处理任务点击（支持 Ctrl/Cmd 多选）
function handleTaskClick(taskId: string, event: MouseEvent) {
  if (event.ctrlKey || event.metaKey) {
    store.toggleTaskSelection(taskId)
  } else {
    store.selectTask(taskId)
  }
}

// 批量操作
async function batchPause() {
  if (selectedProcessingIds.value.length === 0) return
  try {
    await store.batchPauseTasks(selectedProcessingIds.value)
    toast.add({ title: `已暂停 ${selectedProcessingIds.value.length} 个任务`, color: 'warning' })
    store.deselectAllTasks()
  } catch (error: any) {
    toast.add({ title: '批量暂停失败', description: error?.message, color: 'error' })
  }
}

async function batchResume() {
  if (selectedProcessingIds.value.length === 0) return
  try {
    await store.batchResumeTasks(selectedProcessingIds.value)
    toast.add({ title: `已恢复 ${selectedProcessingIds.value.length} 个任务`, color: 'success' })
    store.deselectAllTasks()
  } catch (error: any) {
    toast.add({ title: '批量恢复失败', description: error?.message, color: 'error' })
  }
}

async function batchCancel() {
  if (selectedProcessingIds.value.length === 0) return
  try {
    await store.batchCancelTasks(selectedProcessingIds.value)
    toast.add({ title: `已取消 ${selectedProcessingIds.value.length} 个任务`, color: 'warning' })
    store.deselectAllTasks()
  } catch (error: any) {
    toast.add({ title: '批量取消失败', description: error?.message, color: 'error' })
  }
}

// 选择文件
async function selectFiles() {
  selectingFiles.value = true
  try {
    const selected = await open({
      multiple: true,
      filters: [{ name: 'Excel Files', extensions: ['xlsx', 'xls', 'csv'] }],
    })
    if (selected) {
      const files = Array.isArray(selected) ? selected : [selected]
      const pendingFiles: PendingFile[] = files.map(path => {
        const name = path.split(/[/\\]/).pop() || path
        return {
          id: `pending-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
          path,
          name,
          size: 0,
        }
      })
      store.addPendingFiles(pendingFiles)
      toast.add({ title: `已添加 ${files.length} 个文件`, color: 'success' })
    }
  } catch (error: any) {
    toast.add({ title: '选择文件失败', description: error?.message || String(error), color: 'error' })
  } finally {
    selectingFiles.value = false
  }
}

// 开始处理所有待处理文件
async function startAllPending() {
  if (store.pendingFiles.length === 0) return
  startingAll.value = true
  try {
    const filePaths = store.pendingFiles.map(f => f.path)
    const task = await store.startProcessing(projectId.value, filePaths)
    store.clearPendingFiles()
    toast.add({ title: '已开始批量处理', description: `${filePaths.length} 个文件`, color: 'success' })
  } catch (error: any) {
    toast.add({ title: '批量启动失败', description: error?.message || String(error), color: 'error' })
  } finally {
    startingAll.value = false
  }
}

// 任务控制
async function pauseTask(taskId: string) {
  try {
    await store.pauseTask(taskId)
    toast.add({ title: '任务已暂停', color: 'warning' })
  } catch (error: any) {
    toast.add({ title: '暂停失败', description: error?.message, color: 'error' })
  }
}

async function resumeTask(taskId: string) {
  try {
    await store.resumeTask(taskId)
    toast.add({ title: '任务已恢复', color: 'success' })
  } catch (error: any) {
    toast.add({ title: '恢复失败', description: error?.message, color: 'error' })
  }
}

async function cancelTask(taskId: string) {
  try {
    await store.cancelTask(taskId)
    toast.add({ title: '任务已取消', color: 'warning' })
  } catch (error: any) {
    toast.add({ title: '取消失败', description: error?.message, color: 'error' })
  }
}

// 切换视图模式
function switchViewMode(mode: 'logs' | 'raw' | 'results') {
  detailViewMode.value = mode
  if (mode === 'raw' && store.selectedTask) {
    loadRawData()
  } else if (mode === 'results' && store.selectedTask) {
    loadTaskResults(store.selectedTask.batch_number)
  }
}

// 加载原始数据（从数据库记录中获取）
async function loadRawData() {
  if (!store.selectedTask?.batch_number) {
    toast.add({ title: '没有批次信息', color: 'warning' })
    return
  }

  rawDataLoading.value = true
  rawData.value = null
  rawSheets.value = []

  try {
    // 从数据库查询该批次的所有记录（包含 raw_data）
    const response = await recordsApi.query(projectId.value, {
      batch_number: store.selectedTask.batch_number,
      page: 1,
      page_size: 1000,
    })

    if (response.records.length === 0) {
      toast.add({ title: '没有找到原始数据', color: 'warning' })
      rawDataLoading.value = false
      return
    }

    // 提取 raw_data 构建表格数据
    const rows: any[][] = []
    const sourceFiles = new Set<string>()
    const sourceSheets = new Set<string>()

    // 收集所有字段 ID 作为表头
    const fieldIds = new Set<string>()
    response.records.forEach(record => {
      if (record.data) {
        Object.keys(record.data).forEach(key => fieldIds.add(key))
      }
    })

    // 构建表头行（字段 ID）
    const headerRow = Array.from(fieldIds)
    rows.push(headerRow)

    // 构建数据行
    response.records.forEach(record => {
      if (record.data) {
        const row: any[] = headerRow.map(fieldId => record.data[fieldId] ?? '')
        rows.push(row)
      }
      if (record.source_file) sourceFiles.add(record.source_file)
      if (record.source_sheet) sourceSheets.add(record.source_sheet)
    })

    // 构造 ExcelPreview 格式的数据
    const sheets = Array.from(sourceSheets).map(name => ({
      name,
      row_count: rows.length - 1,
      column_count: headerRow.length,
    }))

    rawData.value = {
      sheets,
      rows,
      sheet_name: sheets[0]?.name || '数据',
    }
    rawSheets.value = sheets
    currentSheet.value = rawData.value.sheet_name
  } catch (error: any) {
    toast.add({ title: '加载原始数据失败', description: error?.message, color: 'error' })
  } finally {
    rawDataLoading.value = false
  }
}

// 加载任务处理结果
async function loadTaskResults(batchNumber: string | null) {
  if (!batchNumber) {
    toast.add({ title: '没有批次信息', color: 'warning' })
    return
  }

  resultsLoading.value = true
  resultsData.value = []

  try {
    const response = await recordsApi.query(projectId.value, {
      batch_number: batchNumber,
      page: 1,
      page_size: 1000, // 增加分页大小
    })
    resultsData.value = response.records
    resultsTotal.value = response.total
  } catch (error: any) {
    toast.add({ title: '加载处理结果失败', description: error?.message, color: 'error' })
  } finally {
    resultsLoading.value = false
  }
}

// 状态图标和颜色
function getStatusIcon(status: string): string {
  const icons: Record<string, string> = {
    pending: 'i-lucide-clock',
    processing: 'i-lucide-loader',
    paused: 'i-lucide-pause-circle',
    completed: 'i-lucide-circle-check',
    cancelled: 'i-lucide-circle-x',
    error: 'i-lucide-circle-alert',
  }
  return icons[status] || 'i-lucide-circle-help'
}

function getStatusIconClass(status: string): string {
  const classes: Record<string, string> = {
    pending: 'w-4 h-4 text-gray-400',
    processing: 'w-4 h-4 text-primary animate-spin',
    paused: 'w-4 h-4 text-yellow-500',
    completed: 'w-4 h-4 text-green-500',
    cancelled: 'w-4 h-4 text-gray-500',
    error: 'w-4 h-4 text-red-500',
  }
  return classes[status] || 'w-4 h-4 text-gray-400'
}

function getStatusColor(status: string): string {
  const colors: Record<string, string> = {
    pending: 'neutral',
    processing: 'primary',
    paused: 'warning',
    completed: 'success',
    cancelled: 'neutral',
    error: 'error',
  }
  return colors[status] || 'neutral'
}

function getStatusText(status: string): string {
  const texts: Record<string, string> = {
    pending: '等待中',
    processing: '处理中',
    paused: '已暂停',
    completed: '已完成',
    cancelled: '已取消',
    error: '失败',
  }
  return texts[status] || status
}

// 获取任务主标题（显示源文件名）
function getTaskTitle(task: { source_files?: string[]; total_files: number; id: string }): string {
  if (task.source_files && task.source_files.length > 0) {
    const firstName = task.source_files[0]
    if (task.source_files.length > 1) {
      return `${firstName} +${task.source_files.length - 1}`
    }
    return firstName
  }
  return `${task.total_files} 个文件`
}

// 获取任务副标题（显示批次号）
function getTaskSubtitle(task: { batch_number?: string | null; id: string }): string {
  return task.batch_number || `ID: ${task.id.slice(0, 8)}`
}

// 日志气泡样式
function getLogBubbleClass(log: { type: string; align?: string; category?: string }): string {
  if (log.align === 'right') {
    // 进度消息（右对齐）
    const classes: Record<string, string> = {
      info: 'bg-info/20 text-info',
      success: 'bg-success/20 text-success',
      warning: 'bg-warning/20 text-warning',
      error: 'bg-error/20 text-error',
    }
    return classes[log.type] || classes.info
  } else {
    // 系统消息（左对齐）
    if (log.category === 'ai') {
      return 'bg-purple-500/10 text-purple-500 border border-purple-500/30'
    }
    if (log.category === 'ai_request') {
      return 'bg-muted text-default border border-default font-mono text-xs whitespace-pre-wrap'
    }
    const classes: Record<string, string> = {
      info: 'bg-elevated text-default border border-default',
      success: 'bg-success/10 text-success border border-success/30',
      warning: 'bg-warning/10 text-warning border border-warning/30',
      error: 'bg-error/10 text-error border border-error/30',
    }
    return classes[log.type] || classes.info
  }
}

// 清除选中任务的日志
function clearSelectedTaskLogs() {
  if (store.selectedTaskId) {
    store.clearTaskLogs(store.selectedTaskId)
  }
}

// 自动滚动日志到底部
watch(
  () => store.selectedLogs.length,
  () => {
    nextTick(() => {
      if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight
      }
    })
  }
)

onMounted(async () => {
  // 启动 Tauri 事件监听
  await store.startEventListener()
  await store.fetchTasks(projectId.value)
  // 启动状态轮询（兜底）
  store.startStatusPolling()
  // 自动选中第一个活动任务
  if (store.activeTasks.length > 0) {
    store.selectTask(store.activeTasks[0].id)
  } else if (store.tasks.length > 0) {
    store.selectTask(store.tasks[0].id)
  }
})

onUnmounted(() => {
  store.stopEventListener()
  store.stopStatusPolling()
})
</script>
