<template>
  <div class="py-4">
    <!-- 工具栏 -->
    <div class="flex justify-between items-center mb-4">
      <div class="text-sm text-gray-500 dark:text-gray-400">
        {{ activeTasks.length > 0 ? `${activeTasks.length} 个任务进行中` : '暂无进行中的任务' }}
      </div>
      <UButton
        icon="i-lucide-upload"
        @click="$emit('upload')"
      >
        导入文件
      </UButton>
    </div>

    <!-- 空状态 -->
    <div
      v-if="tasks.length === 0"
      class="text-center py-12 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700"
    >
      <UIcon name="i-lucide-file-up" class="w-12 h-12 mx-auto text-gray-400 mb-4" />
      <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">还没有处理任务</h3>
      <p class="text-gray-500 dark:text-gray-400 mb-6">导入 Excel 文件开始数据处理</p>
      <UButton icon="i-lucide-upload" @click="$emit('upload')">
        导入文件
      </UButton>
    </div>

    <!-- 任务列表 -->
    <div v-else class="space-y-4">
      <div
        v-for="task in sortedTasks"
        :key="task.id"
        class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4"
      >
        <!-- 任务头部 -->
        <div class="flex justify-between items-start mb-3">
          <div>
            <div class="flex items-center gap-2">
              <UIcon :name="getStatusIcon(task.status)" :class="getStatusIconClass(task.status)" />
              <span class="font-medium text-gray-900 dark:text-white">
                {{ task.file_name }}
              </span>
              <UBadge :color="getStatusColor(task.status)" variant="subtle" size="xs">
                {{ getStatusText(task.status) }}
              </UBadge>
            </div>
            <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">
              {{ task.sheet_name }} · 开始于 {{ formatTime(task.started_at) }}
            </div>
          </div>
          <div class="flex gap-2">
            <UButton
              v-if="task.status === 'processing'"
              icon="i-lucide-pause"
              color="neutral"
              variant="ghost"
              size="xs"
              @click="pauseTask(task.id)"
            >
              暂停
            </UButton>
            <UButton
              v-if="task.status === 'paused'"
              icon="i-lucide-play"
              color="neutral"
              variant="ghost"
              size="xs"
              @click="resumeTask(task.id)"
            >
              继续
            </UButton>
            <UButton
              v-if="task.status === 'processing' || task.status === 'paused'"
              icon="i-lucide-square"
              color="error"
              variant="ghost"
              size="xs"
              @click="cancelTask(task.id)"
            >
              取消
            </UButton>
            <UButton
              v-if="task.status === 'completed' || task.status === 'cancelled' || task.status === 'failed'"
              icon="i-lucide-refresh-cw"
              color="neutral"
              variant="ghost"
              size="xs"
              @click="retryTask(task.id)"
            >
              重试
            </UButton>
          </div>
        </div>

        <!-- 进度条 -->
        <div v-if="task.status === 'processing' || task.status === 'paused'" class="mb-3">
          <div class="flex justify-between text-sm text-gray-500 dark:text-gray-400 mb-1">
            <span>{{ task.processed_rows }} / {{ task.total_rows }} 行</span>
            <span>{{ task.progress }}%</span>
          </div>
          <UProgress :value="task.progress" :color="task.status === 'paused' ? 'warning' : 'primary'" />
        </div>

        <!-- 统计信息 -->
        <div v-if="task.status === 'completed'" class="grid grid-cols-3 gap-4 text-sm">
          <div>
            <span class="text-gray-500 dark:text-gray-400">成功：</span>
            <span class="text-green-600 dark:text-green-400 font-medium">{{ task.success_count }}</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400">失败：</span>
            <span class="text-red-600 dark:text-red-400 font-medium">{{ task.error_count }}</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400">耗时：</span>
            <span class="text-gray-900 dark:text-white font-medium">{{ formatDuration(task.duration) }}</span>
          </div>
        </div>

        <!-- 错误信息 -->
        <div v-if="task.status === 'failed' && task.error_message" class="mt-3 p-2 bg-red-50 dark:bg-red-900/20 rounded text-sm text-red-600 dark:text-red-400">
          {{ task.error_message }}
        </div>
      </div>
    </div>

    <!-- 实时进度事件 -->
    <div v-if="activeTasks.length > 0" class="mt-6">
      <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">实时日志</h3>
      <div class="bg-gray-900 dark:bg-gray-950 rounded-lg p-4 max-h-64 overflow-y-auto font-mono text-sm">
        <div
          v-for="(log, index) in recentLogs"
          :key="index"
          class="text-gray-300"
          :class="{
            'text-green-400': log.type === 'success',
            'text-yellow-400': log.type === 'warning',
            'text-red-400': log.type === 'error',
          }"
        >
          <span class="text-gray-500">[{{ log.time }}]</span> {{ log.message }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import type { ProcessingTask } from '~/types'

const route = useRoute()
const projectId = computed(() => Number(route.params.id))

defineEmits<{
  upload: []
}>()

// 文件选择
function selectFiles() {
  // TODO: 调用 Tauri 文件选择对话框
  console.log('Select files for project:', projectId.value)
}

// 任务列表
const tasks = ref<ProcessingTask[]>([])
const recentLogs = ref<Array<{ time: string; message: string; type: string }>>([])

// 计算属性
const activeTasks = computed(() =>
  tasks.value.filter(t => t.status === 'processing' || t.status === 'paused')
)

const sortedTasks = computed(() =>
  [...tasks.value].sort((a, b) =>
    new Date(b.started_at).getTime() - new Date(a.started_at).getTime()
  )
)

// 获取状态图标
function getStatusIcon(status: string): string {
  const icons: Record<string, string> = {
    pending: 'i-lucide-clock',
    processing: 'i-lucide-refresh-cw',
    paused: 'i-lucide-pause-circle',
    completed: 'i-lucide-circle-check',
    cancelled: 'i-lucide-circle-x',
    failed: 'i-lucide-circle-alert',
  }
  return icons[status] || 'i-lucide-circle-help'
}

function getStatusIconClass(status: string): string {
  const classes: Record<string, string> = {
    pending: 'w-5 h-5 text-gray-400',
    processing: 'w-5 h-5 text-primary animate-spin',
    paused: 'w-5 h-5 text-yellow-500',
    completed: 'w-5 h-5 text-green-500',
    cancelled: 'w-5 h-5 text-gray-500',
    failed: 'w-5 h-5 text-red-500',
  }
  return classes[status] || 'w-5 h-5 text-gray-400'
}

function getStatusColor(status: string): string {
  const colors: Record<string, string> = {
    pending: 'neutral',
    processing: 'primary',
    paused: 'warning',
    completed: 'success',
    cancelled: 'neutral',
    failed: 'error',
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
    failed: '失败',
  }
  return texts[status] || status
}

// 格式化时间
function formatTime(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}秒`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}分${seconds % 60}秒`
  return `${Math.floor(seconds / 3600)}时${Math.floor((seconds % 3600) / 60)}分`
}

// 任务操作
async function pauseTask(taskId: string) {
  try {
    // await processingApi.pause(taskId)
    console.log('Pause task:', taskId)
  } catch (error) {
    console.error('Failed to pause task:', error)
  }
}

async function resumeTask(taskId: string) {
  try {
    // await processingApi.resume(taskId)
    console.log('Resume task:', taskId)
  } catch (error) {
    console.error('Failed to resume task:', error)
  }
}

async function cancelTask(taskId: string) {
  try {
    // await processingApi.cancel(taskId)
    console.log('Cancel task:', taskId)
  } catch (error) {
    console.error('Failed to cancel task:', error)
  }
}

async function retryTask(taskId: string) {
  try {
    // await processingApi.retry(taskId)
    console.log('Retry task:', taskId)
  } catch (error) {
    console.error('Failed to retry task:', error)
  }
}

// WebSocket 连接
let ws: WebSocket | null = null

function connectWebSocket() {
  // TODO: 连接 WebSocket 接收实时进度
  // ws = new WebSocket(`ws://127.0.0.1:8000/ws/processing/${props.projectId}`)
  // ws.onmessage = (event) => {
  //   const data = JSON.parse(event.data)
  //   handleProgressUpdate(data)
  // }
}

function disconnectWebSocket() {
  if (ws) {
    ws.close()
    ws = null
  }
}

function handleProgressUpdate(data: any) {
  // 更新任务进度
  const taskIndex = tasks.value.findIndex(t => t.id === data.task_id)
  if (taskIndex !== -1) {
    tasks.value[taskIndex] = { ...tasks.value[taskIndex], ...data }
  }

  // 添加日志
  recentLogs.value.push({
    time: new Date().toLocaleTimeString('zh-CN'),
    message: data.message || `处理进度: ${data.progress}%`,
    type: data.type || 'info',
  })

  // 保持最近 100 条日志
  if (recentLogs.value.length > 100) {
    recentLogs.value.shift()
  }
}

// 加载任务列表
async function loadTasks() {
  try {
    // tasks.value = await processingApi.list(props.projectId)
    // 临时模拟数据
    tasks.value = []
  } catch (error) {
    console.error('Failed to load tasks:', error)
  }
}

onMounted(() => {
  loadTasks()
  connectWebSocket()
})

onUnmounted(() => {
  disconnectWebSocket()
})
</script>
