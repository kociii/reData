<template>
  <div class="py-4 flex flex-col h-full">
    <!-- 工具栏 -->
    <div class="flex justify-between items-center mb-4">
      <div class="text-sm text-gray-500 dark:text-gray-400">
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
      class="text-center py-16 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700"
    >
      <UIcon name="i-lucide-file-up" class="w-12 h-12 mx-auto text-gray-400 mb-4" />
      <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">还没有处理任务</h3>
      <p class="text-gray-500 dark:text-gray-400 mb-6">导入 Excel 文件开始数据处理</p>
      <UButton icon="i-lucide-upload" :loading="selectingFiles" @click="selectFiles">
        导入文件
      </UButton>
    </div>

    <!-- 主内容区：左右分栏 -->
    <div v-else class="flex gap-4 flex-1 min-h-0">
      <!-- 左侧面板 -->
      <div class="w-72 flex-shrink-0 flex flex-col gap-4 overflow-y-auto">
        <!-- 待处理文件 -->
        <div v-if="store.hasPendingFiles">
          <div class="flex justify-between items-center mb-2">
            <h3 class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              待处理 ({{ store.pendingFiles.length }})
            </h3>
            <UButton
              variant="ghost" size="xs" color="neutral" icon="i-lucide-trash-2"
              @click="store.clearPendingFiles"
            />
          </div>
          <div class="space-y-1">
            <div
              v-for="file in store.pendingFiles" :key="file.id"
              class="flex items-center gap-2 p-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 text-sm"
            >
              <UIcon name="i-lucide-file-spreadsheet" class="w-4 h-4 text-green-500 flex-shrink-0" />
              <span class="truncate flex-1 text-gray-900 dark:text-white">{{ file.name }}</span>
              <UButton
                size="xs" variant="ghost" color="neutral" icon="i-lucide-x"
                @click="store.removePendingFile(file.id)"
              />
            </div>
          </div>
        </div>

        <!-- 任务列表 -->
        <div>
          <h3 class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
            任务列表
          </h3>
          <div class="space-y-1">
            <div
              v-for="task in sortedTasks" :key="task.id"
              class="flex items-center gap-2 p-2.5 rounded-lg border text-sm cursor-pointer transition-colors"
              :class="task.id === store.selectedTaskId
                ? 'bg-primary/10 border-primary/30 dark:bg-primary/10'
                : 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-750'"
              @click="store.selectTask(task.id)"
            >
              <UIcon :name="getStatusIcon(task.status)" :class="getStatusIconClass(task.status)" class="flex-shrink-0" />
              <div class="min-w-0 flex-1">
                <div class="font-medium text-gray-900 dark:text-white truncate text-xs">
                  {{ task.batch_number || `任务 ${task.id.slice(0, 8)}` }}
                </div>
                <div class="text-xs text-gray-500 dark:text-gray-400">
                  {{ task.total_files }} 文件 · {{ getStatusText(task.status) }}
                </div>
              </div>
              <UBadge :color="getStatusColor(task.status)" variant="subtle" size="xs">
                {{ task.success_count }}
              </UBadge>
            </div>
          </div>
        </div>
      </div>

      <!-- 右侧面板 -->
      <div class="flex-1 flex flex-col gap-4 min-w-0 overflow-y-auto">
        <template v-if="store.selectedTask">
          <!-- 进度阶段 -->
          <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
            <ProcessingStages :stages="store.selectedStages" />
          </div>

          <!-- 统计卡片 -->
          <div class="grid grid-cols-4 gap-3">
            <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-3 text-center">
              <div class="text-2xl font-bold text-gray-900 dark:text-white">{{ store.selectedTask.total_files }}</div>
              <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">文件数</div>
            </div>
            <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-3 text-center">
              <div class="text-2xl font-bold text-green-600 dark:text-green-400">{{ store.selectedTask.success_count }}</div>
              <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">成功</div>
            </div>
            <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-3 text-center">
              <div class="text-2xl font-bold text-red-600 dark:text-red-400">{{ store.selectedTask.error_count }}</div>
              <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">失败</div>
            </div>
            <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-3 text-center">
              <div class="text-2xl font-bold text-gray-900 dark:text-white text-sm">{{ store.selectedTask.batch_number || '-' }}</div>
              <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">批次</div>
            </div>
          </div>

          <!-- 进度条（处理中时显示） -->
          <div
            v-if="store.selectedTask.status === 'processing' || store.selectedTask.status === 'paused'"
            class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4"
          >
            <div class="flex justify-between text-sm text-gray-500 dark:text-gray-400 mb-2">
              <span>{{ store.selectedTask.processed_rows }} / {{ store.selectedTask.total_rows }} 行</span>
              <span>{{ store.selectedTask.progress || 0 }}%</span>
            </div>
            <UProgress
              :value="store.selectedTask.progress || 0"
              :color="store.selectedTask.status === 'paused' ? 'warning' : 'primary'"
            />
            <div class="flex gap-2 mt-3 justify-end">
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
          </div>

          <!-- 操作按钮（已完成时） -->
          <div v-if="store.selectedTask.status === 'completed'" class="flex gap-2">
            <UButton
              icon="i-lucide-table" color="primary" variant="soft"
              @click="viewResults(store.selectedTask)"
            >
              查看结果
            </UButton>
          </div>

          <!-- 处理日志 -->
          <div class="flex-1 min-h-0 flex flex-col">
            <div class="flex justify-between items-center mb-2">
              <h3 class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">处理日志</h3>
              <UButton
                variant="ghost" size="xs" color="neutral" icon="i-lucide-trash-2"
                @click="store.clearLogs"
              />
            </div>
            <div
              ref="logContainer"
              class="bg-gray-900 dark:bg-gray-950 rounded-lg p-3 flex-1 overflow-y-auto font-mono text-xs min-h-40 max-h-64"
            >
              <div v-if="store.logs.length === 0" class="text-gray-500">等待日志...</div>
              <div
                v-for="(log, index) in store.logs" :key="index"
                class="leading-5"
                :class="{
                  'text-gray-300': log.type === 'info',
                  'text-green-400': log.type === 'success',
                  'text-yellow-400': log.type === 'warning',
                  'text-red-400': log.type === 'error',
                }"
              >
                <span class="text-gray-500">[{{ log.time }}]</span> {{ log.message }}
              </div>
            </div>
          </div>
        </template>

        <!-- 未选中任务时的提示 -->
        <div v-else class="flex-1 flex items-center justify-center text-gray-400 dark:text-gray-500">
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
import { open } from '@tauri-apps/plugin-dialog'
import { useProcessingStore } from '~/stores/processing'
import type { PendingFile } from '~/types'

const route = useRoute()
const router = useRouter()
const toast = useToast()
const store = useProcessingStore()

const projectId = computed(() => Number(route.params.id))

const selectingFiles = ref(false)
const startingAll = ref(false)
const logContainer = ref<HTMLElement | null>(null)

// 按时间倒序排列任务，处理中的排前面
const sortedTasks = computed(() =>
  [...store.tasks].sort((a, b) => {
    if (a.status === 'processing' && b.status !== 'processing') return -1
    if (b.status === 'processing' && a.status !== 'processing') return 1
    return (b.batch_number || b.id).localeCompare(a.batch_number || a.id)
  })
)

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
    store.connectWebSocket(task.task_id)
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

function viewResults(task: { batch_number: string | null }) {
  if (task.batch_number) {
    router.push(`/project/${projectId.value}/results?batch=${task.batch_number}`)
  } else {
    router.push(`/project/${projectId.value}/results`)
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

// 自动滚动日志到底部
watch(
  () => store.logs.length,
  () => {
    nextTick(() => {
      if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight
      }
    })
  }
)

// 监听活动任务，自动连接 WebSocket
watch(
  () => store.activeTasks,
  (activeTasks) => {
    activeTasks.forEach(task => {
      if (!store.wsConnections.has(task.id)) {
        store.connectWebSocket(task.id)
      }
    })
  },
  { deep: true }
)

onMounted(async () => {
  await store.fetchTasks(projectId.value)
  // 为所有进行中的任务连接 WebSocket
  store.activeTasks.forEach(task => {
    store.connectWebSocket(task.id)
  })
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
  store.disconnectAllWebSockets()
  store.stopStatusPolling()
})
</script>
