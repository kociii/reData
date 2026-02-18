<template>
  <div class="flex flex-col h-full p-6">
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
      <UButton
        icon="i-lucide-upload"
        :loading="selectingFiles"
        @click="selectFiles"
      >
        导入文件并开始处理
      </UButton>
    </div>

    <!-- 空状态 -->
    <div
      v-if="store.tasks.length === 0"
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
    <div v-else class="flex gap-4 flex-1 min-h-0">
      <!-- 左侧任务列表 -->
      <div class="w-64 flex-shrink-0 flex flex-col gap-4 overflow-y-auto">
        <!-- 处理中分组 -->
        <div v-if="store.activeTasks.length > 0">
          <div class="flex items-center gap-2 px-1 mb-2">
            <span class="text-xs font-medium text-primary uppercase tracking-wider">处理中</span>
            <UBadge color="primary" variant="subtle" size="xs">{{ store.activeTasks.length }}</UBadge>
          </div>
          <div class="space-y-1">
            <div
              v-for="task in store.activeTasks"
              :key="task.taskId"
              class="flex items-center gap-2 p-2 rounded-lg border text-sm cursor-pointer transition-colors"
              :class="task.taskId === store.selectedTaskId
                ? 'bg-primary/10 border-primary/30'
                : 'bg-muted border-default hover:bg-accented'"
              @click="store.selectTask(task.taskId)"
            >
              <UIcon
                :name="task.phase === 'paused' ? 'i-lucide-pause-circle' : 'i-lucide-loader'"
                :class="task.phase === 'paused' ? 'w-4 h-4 text-warning flex-shrink-0' : 'w-4 h-4 text-primary animate-spin flex-shrink-0'"
              />
              <div class="min-w-0 flex-1">
                <div class="font-medium text-highlighted truncate text-xs">{{ getTaskTitle(task) }}</div>
                <div class="text-xs text-muted">
                  {{ task.phase === 'paused' ? '已暂停' : '处理中' }} · 成功 {{ task.successCount }}
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 已完成分组 -->
        <div v-if="store.completedTasks.length > 0">
          <div class="flex items-center gap-2 px-1 mb-2">
            <span class="text-xs font-medium text-muted uppercase tracking-wider">已完成</span>
            <UBadge color="neutral" variant="subtle" size="xs">{{ store.completedTasks.length }}</UBadge>
          </div>
          <div class="space-y-1">
            <div
              v-for="task in store.completedTasks"
              :key="task.taskId"
              class="flex items-center gap-2 p-2 rounded-lg border text-sm cursor-pointer transition-colors"
              :class="task.taskId === store.selectedTaskId
                ? 'bg-primary/10 border-primary/30'
                : 'bg-muted border-default hover:bg-accented'"
              @click="store.selectTask(task.taskId)"
            >
              <UIcon
                :name="getTaskPhaseIcon(task.phase)"
                :class="getTaskPhaseIconClass(task.phase)"
                class="flex-shrink-0 w-4 h-4"
              />
              <div class="min-w-0 flex-1">
                <div class="font-medium text-highlighted truncate text-xs">{{ getTaskTitle(task) }}</div>
                <div class="text-xs text-muted">
                  {{ getTaskPhaseText(task.phase) }} · 成功 {{ task.successCount }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 右侧任务详情 -->
      <div class="flex-1 flex flex-col gap-4 min-w-0 overflow-y-auto">
        <template v-if="store.selectedTask">
          <!-- 任务头部：批次号 + 状态 + 控制按钮 -->
          <div class="flex items-center justify-between p-3 bg-elevated rounded-lg border border-default">
            <div>
              <div class="font-medium text-highlighted text-sm">
                {{ store.selectedTask.batchNumber || `任务 ${store.selectedTask.taskId.slice(0, 8)}` }}
              </div>
              <div class="text-xs text-muted flex items-center gap-1.5 mt-0.5">
                <UIcon
                  :name="getTaskPhaseIcon(store.selectedTask.phase)"
                  :class="getTaskPhaseIconClass(store.selectedTask.phase)"
                  class="w-3.5 h-3.5"
                />
                {{ getTaskPhaseText(store.selectedTask.phase) }}
              </div>
            </div>
            <div
              v-if="store.selectedTask.phase === 'processing' || store.selectedTask.phase === 'paused'"
              class="flex gap-2"
            >
              <UButton
                v-if="store.selectedTask.phase === 'processing'"
                icon="i-lucide-pause"
                color="neutral"
                variant="ghost"
                size="xs"
                @click="pauseTask(store.selectedTask.taskId)"
              >
                暂停
              </UButton>
              <UButton
                v-if="store.selectedTask.phase === 'paused'"
                icon="i-lucide-play"
                color="neutral"
                variant="ghost"
                size="xs"
                @click="resumeTask(store.selectedTask.taskId)"
              >
                继续
              </UButton>
              <UButton
                icon="i-lucide-square"
                color="error"
                variant="ghost"
                size="xs"
                @click="cancelTask(store.selectedTask.taskId)"
              >
                取消
              </UButton>
            </div>
          </div>

          <!-- 汇总进度 -->
          <div class="p-3 bg-elevated rounded-lg border border-default">
            <div class="flex items-center justify-between mb-2">
              <span class="text-xs font-medium text-muted">总进度</span>
              <span class="text-xs text-muted">
                成功 {{ store.selectedTask.successCount }}
                <template v-if="store.selectedTask.errorCount > 0">
                  · 失败 {{ store.selectedTask.errorCount }}
                </template>
              </span>
            </div>
            <UProgress
              :model-value="taskProgressPercent"
              size="sm"
              class="w-full"
            />
            <div class="text-xs text-muted mt-1">
              {{ store.selectedTask.processedRows }} 行已处理
              <template v-if="store.selectedTask.phase === 'completed'">
                · 完成
              </template>
            </div>
          </div>

          <!-- 文件/Sheet 进度树 -->
          <div class="bg-elevated rounded-lg border border-default overflow-hidden">
            <!-- 无文件时的占位 -->
            <div
              v-if="store.selectedTask.files.length === 0"
              class="p-6 text-center text-muted text-sm"
            >
              <template v-if="['completed', 'cancelled', 'error'].includes(store.selectedTask.phase)">
                <UIcon name="i-lucide-file-x" class="w-5 h-5 mx-auto mb-2 text-dimmed" />
                暂无文件明细记录
              </template>
              <template v-else>
                <UIcon name="i-lucide-loader" class="w-5 h-5 mx-auto mb-2 animate-spin text-primary" />
                等待文件处理...
              </template>
            </div>

            <!-- 文件列表 -->
            <div
              v-for="(file, fileIdx) in store.selectedTask.files"
              :key="file.fileName"
              :class="fileIdx < store.selectedTask.files.length - 1 ? 'border-b border-default' : ''"
            >
              <!-- 文件行 -->
              <div class="flex items-center gap-3 px-4 py-3">
                <UIcon
                  :name="getFilePhaseIcon(file.phase)"
                  :class="getFilePhaseIconClass(file.phase)"
                  class="flex-shrink-0 w-4 h-4"
                />
                <div class="min-w-0 flex-1">
                  <div class="text-sm font-medium text-highlighted truncate">{{ file.fileName }}</div>
                  <div class="text-xs text-muted mt-0.5">
                    <template v-if="file.phase === 'done'">
                      完成 · 成功 {{ file.successCount }} 行
                      <template v-if="file.errorCount > 0">· 失败 {{ file.errorCount }}</template>
                    </template>
                    <template v-else-if="file.phase === 'error'">
                      处理失败
                    </template>
                    <template v-else-if="file.phase === 'processing'">
                      处理中...
                    </template>
                    <template v-else>
                      等待中
                    </template>
                  </div>
                </div>
                <UBadge
                  :color="getFilePhaseBadgeColor(file.phase)"
                  variant="subtle"
                  size="xs"
                >
                  {{ getFilePhaseBadgeText(file.phase) }}
                </UBadge>
              </div>

              <!-- Sheet 列表 -->
              <div v-if="file.sheets.length > 0" class="pb-2 space-y-0.5 bg-muted/30">
                <div
                  v-for="sheet in file.sheets"
                  :key="sheet.sheetName"
                  class="flex items-start gap-2 px-4 py-2"
                >
                  <!-- 连接线 -->
                  <div class="flex flex-col items-center flex-shrink-0 mt-1" style="width: 20px;">
                    <div class="w-px h-2 bg-default" />
                    <div class="w-3 h-px bg-default" />
                  </div>

                  <!-- Sheet 状态图标 -->
                  <UIcon
                    :name="getSheetPhaseIcon(sheet.phase)"
                    :class="getSheetPhaseIconClass(sheet.phase)"
                    class="flex-shrink-0 w-3.5 h-3.5 mt-0.5"
                  />

                  <!-- Sheet 信息 -->
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2 flex-wrap">
                      <span class="text-xs font-medium text-default">{{ sheet.sheetName }}</span>
                      <!-- AI 置信度标签 -->
                      <span
                        v-if="sheet.aiConfidence !== null"
                        class="text-xs text-muted"
                      >
                        [AI {{ (sheet.aiConfidence * 100).toFixed(0) }}%
                        <template v-if="sheet.mappingCount !== null">· {{ sheet.mappingCount }} 字段</template>
                        ]
                      </span>
                    </div>

                    <!-- 状态描述 -->
                    <div class="mt-0.5">
                      <!-- AI 识别中 -->
                      <template v-if="sheet.phase === 'ai_analyzing'">
                        <div class="flex items-center gap-1.5 text-xs text-muted">
                          <UIcon name="i-lucide-sparkles" class="w-3 h-3 text-primary animate-pulse" />
                          AI 识别列映射中...
                        </div>
                      </template>

                      <!-- 导入中：不定进度动画 -->
                      <template v-else-if="sheet.phase === 'importing'">
                        <div class="flex items-center gap-2 mt-1">
                          <UProgress
                            size="xs"
                            class="flex-1 max-w-32"
                          />
                          <span class="text-xs text-muted">导入中...</span>
                        </div>
                      </template>

                      <!-- 完成 -->
                      <template v-else-if="sheet.phase === 'done'">
                        <span class="text-xs text-success">完成</span>
                      </template>

                      <!-- 错误 -->
                      <template v-else-if="sheet.phase === 'error'">
                        <span class="text-xs text-error">{{ sheet.errorMessage || '处理失败' }}</span>
                      </template>

                      <!-- 等待 -->
                      <template v-else>
                        <span class="text-xs text-dimmed">等待中</span>
                      </template>
                    </div>
                  </div>

                  <!-- Sheet 状态徽章 -->
                  <UBadge
                    :color="getSheetPhaseBadgeColor(sheet.phase)"
                    variant="subtle"
                    size="xs"
                    class="flex-shrink-0 mt-0.5"
                  >
                    {{ getSheetPhaseBadgeText(sheet.phase) }}
                  </UBadge>
                </div>
              </div>
            </div>
          </div>

          <!-- 查看结果按钮（仅完成时显示） -->
          <div v-if="store.selectedTask.phase === 'completed'" class="flex justify-end">
            <UButton
              icon="i-lucide-table-2"
              color="success"
              @click="viewResults"
            >
              查看处理结果
            </UButton>
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
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { open } from '@tauri-apps/plugin-dialog'
import { useProcessingStore } from '~/stores/processing'
import type { TaskProgress, TaskPhase, FilePhase, SheetPhase } from '~/types'

const route = useRoute()
const router = useRouter()
const toast = useToast()
const store = useProcessingStore()

const projectId = computed(() => Number(route.params.id))
const selectingFiles = ref(false)

// 任务总进度百分比
const taskProgressPercent = computed(() => {
  const task = store.selectedTask
  if (!task) return 0
  if (task.phase === 'completed') return 100
  if (!task.totalRows || task.totalRows === 0) return 0
  return Math.min(Math.round((task.processedRows / task.totalRows) * 100), 99)
})

// ── 辅助函数 ───────────────────────────────────────────────────────────────────

function getTaskTitle(task: TaskProgress): string {
  if (task.sourceFiles.length > 0) {
    const first = task.sourceFiles[0]
    if (task.sourceFiles.length > 1) {
      return `${first} +${task.sourceFiles.length - 1}`
    }
    return first
  }
  return `任务 ${task.taskId.slice(0, 8)}`
}

function getTaskPhaseIcon(phase: TaskPhase): string {
  const icons: Record<TaskPhase, string> = {
    starting: 'i-lucide-clock',
    processing: 'i-lucide-loader',
    paused: 'i-lucide-pause-circle',
    completed: 'i-lucide-circle-check',
    cancelled: 'i-lucide-circle-x',
    error: 'i-lucide-circle-alert',
  }
  return icons[phase] ?? 'i-lucide-circle'
}

function getTaskPhaseIconClass(phase: TaskPhase): string {
  const classes: Record<TaskPhase, string> = {
    starting: 'text-muted',
    processing: 'text-primary animate-spin',
    paused: 'text-warning',
    completed: 'text-success',
    cancelled: 'text-muted',
    error: 'text-error',
  }
  return classes[phase] ?? 'text-muted'
}

function getTaskPhaseText(phase: TaskPhase): string {
  const texts: Record<TaskPhase, string> = {
    starting: '启动中',
    processing: '处理中',
    paused: '已暂停',
    completed: '已完成',
    cancelled: '已取消',
    error: '失败',
  }
  return texts[phase] ?? phase
}

function getFilePhaseIcon(phase: FilePhase): string {
  const icons: Record<FilePhase, string> = {
    waiting: 'i-lucide-file-spreadsheet',
    processing: 'i-lucide-file-spreadsheet',
    done: 'i-lucide-file-check',
    error: 'i-lucide-file-x',
  }
  return icons[phase] ?? 'i-lucide-file'
}

function getFilePhaseIconClass(phase: FilePhase): string {
  const classes: Record<FilePhase, string> = {
    waiting: 'text-muted',
    processing: 'text-primary',
    done: 'text-success',
    error: 'text-error',
  }
  return classes[phase] ?? 'text-muted'
}

function getFilePhaseBadgeColor(phase: FilePhase): string {
  const colors: Record<FilePhase, string> = {
    waiting: 'neutral',
    processing: 'primary',
    done: 'success',
    error: 'error',
  }
  return colors[phase] ?? 'neutral'
}

function getFilePhaseBadgeText(phase: FilePhase): string {
  const texts: Record<FilePhase, string> = {
    waiting: '等待',
    processing: '处理中',
    done: '完成',
    error: '失败',
  }
  return texts[phase] ?? phase
}

function getSheetPhaseIcon(phase: SheetPhase): string {
  const icons: Record<SheetPhase, string> = {
    waiting: 'i-lucide-clock',
    ai_analyzing: 'i-lucide-sparkles',
    importing: 'i-lucide-loader',
    done: 'i-lucide-check-circle',
    error: 'i-lucide-alert-circle',
  }
  return icons[phase] ?? 'i-lucide-circle'
}

function getSheetPhaseIconClass(phase: SheetPhase): string {
  const classes: Record<SheetPhase, string> = {
    waiting: 'text-dimmed',
    ai_analyzing: 'text-primary animate-pulse',
    importing: 'text-primary animate-spin',
    done: 'text-success',
    error: 'text-error',
  }
  return classes[phase] ?? 'text-muted'
}

function getSheetPhaseBadgeColor(phase: SheetPhase): string {
  const colors: Record<SheetPhase, string> = {
    waiting: 'neutral',
    ai_analyzing: 'primary',
    importing: 'info',
    done: 'success',
    error: 'error',
  }
  return colors[phase] ?? 'neutral'
}

function getSheetPhaseBadgeText(phase: SheetPhase): string {
  const texts: Record<SheetPhase, string> = {
    waiting: '等待',
    ai_analyzing: 'AI 识别',
    importing: '导入中',
    done: '完成',
    error: '失败',
  }
  return texts[phase] ?? phase
}

// ── 操作 ───────────────────────────────────────────────────────────────────────

// 选择文件并直接开始处理（无待处理队列）
async function selectFiles() {
  selectingFiles.value = true
  try {
    const selected = await open({
      multiple: true,
      filters: [{ name: 'Excel Files', extensions: ['xlsx', 'xls', 'csv'] }],
    })
    if (selected) {
      const files = Array.isArray(selected) ? selected : [selected]
      await store.startProcessing(projectId.value, files)
      toast.add({ title: `已开始处理 ${files.length} 个文件`, color: 'success' })
    }
  }
  catch (error: any) {
    toast.add({ title: '启动处理失败', description: error?.message || String(error), color: 'error' })
  }
  finally {
    selectingFiles.value = false
  }
}

async function pauseTask(taskId: string) {
  try {
    await store.pauseTask(taskId)
    toast.add({ title: '任务已暂停', color: 'warning' })
  }
  catch (error: any) {
    toast.add({ title: '暂停失败', description: error?.message, color: 'error' })
  }
}

async function resumeTask(taskId: string) {
  try {
    await store.resumeTask(taskId)
    toast.add({ title: '任务已恢复', color: 'success' })
  }
  catch (error: any) {
    toast.add({ title: '恢复失败', description: error?.message, color: 'error' })
  }
}

async function cancelTask(taskId: string) {
  try {
    await store.cancelTask(taskId)
    toast.add({ title: '任务已取消', color: 'warning' })
  }
  catch (error: any) {
    toast.add({ title: '取消失败', description: error?.message, color: 'error' })
  }
}

function viewResults() {
  const batch = store.selectedTask?.batchNumber
  const basePath = `/project/${projectId.value}/results`
  router.push(batch ? `${basePath}?batch=${encodeURIComponent(batch)}` : basePath)
}

// ── 生命周期 ───────────────────────────────────────────────────────────────────

onMounted(async () => {
  await store.startEventListener()
  await store.fetchTasks(projectId.value)
  store.startStatusPolling()
  // 自动选中第一个活动任务，或第一个历史任务
  if (store.activeTasks.length > 0) {
    store.selectTask(store.activeTasks[0].taskId)
  }
  else if (store.tasks.length > 0) {
    store.selectTask(store.tasks[0].taskId)
  }
})

onUnmounted(() => {
  store.stopEventListener()
  store.stopStatusPolling()
})
</script>
