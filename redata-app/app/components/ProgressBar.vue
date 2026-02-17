<template>
  <div class="w-full">
    <!-- 进度条 -->
    <div class="relative h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
      <div
        class="absolute top-0 left-0 h-full transition-all duration-300 ease-out rounded-full"
        :class="progressBarClass"
        :style="{ width: `${clampedProgress}%` }"
      />
    </div>

    <!-- 进度信息 -->
    <div v-if="showInfo" class="flex justify-between items-center mt-2 text-sm">
      <span class="text-gray-500 dark:text-gray-400">
        <template v-if="current !== undefined && total !== undefined">
          {{ current }} / {{ total }} {{ unit }}
        </template>
        <template v-else>
          {{ clampedProgress }}%
        </template>
      </span>
      <span v-if="estimatedTime" class="text-gray-500 dark:text-gray-400">
        预计剩余: {{ formatTime(estimatedTime) }}
      </span>
    </div>

    <!-- 状态标签 -->
    <div v-if="status" class="flex items-center gap-2 mt-2">
      <UIcon :name="statusIcon" :class="statusIconClass" />
      <span class="text-sm" :class="statusTextClass">{{ statusText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    progress: number
    current?: number
    total?: number
    unit?: string
    estimatedTime?: number
    status?: 'processing' | 'paused' | 'completed' | 'failed'
    showInfo?: boolean
    color?: 'primary' | 'success' | 'warning' | 'error'
  }>(),
  {
    unit: '项',
    showInfo: true,
    color: 'primary',
  }
)

// 确保进度在 0-100 之间
const clampedProgress = computed(() => Math.min(100, Math.max(0, props.progress)))

// 进度条颜色
const progressBarClass = computed(() => {
  if (props.status === 'paused') return 'bg-yellow-500'
  if (props.status === 'completed') return 'bg-green-500'
  if (props.status === 'failed') return 'bg-red-500'
  return 'bg-primary-500'
})

// 状态图标
const statusIcon = computed(() => {
  const icons: Record<string, string> = {
    processing: 'i-lucide-refresh-cw',
    paused: 'i-lucide-pause-circle',
    completed: 'i-lucide-circle-check',
    failed: 'i-lucide-circle-alert',
  }
  return icons[props.status || 'processing'] || 'i-lucide-refresh-cw'
})

const statusIconClass = computed(() => {
  const classes: Record<string, string> = {
    processing: 'w-4 h-4 text-primary animate-spin',
    paused: 'w-4 h-4 text-yellow-500',
    completed: 'w-4 h-4 text-green-500',
    failed: 'w-4 h-4 text-red-500',
  }
  return classes[props.status || 'processing'] || 'w-4 h-4 text-gray-400'
})

const statusText = computed(() => {
  const texts: Record<string, string> = {
    processing: '处理中...',
    paused: '已暂停',
    completed: '已完成',
    failed: '失败',
  }
  return texts[props.status || 'processing'] || ''
})

const statusTextClass = computed(() => {
  const classes: Record<string, string> = {
    processing: 'text-primary-600 dark:text-primary-400',
    paused: 'text-yellow-600 dark:text-yellow-400',
    completed: 'text-green-600 dark:text-green-400',
    failed: 'text-red-600 dark:text-red-400',
  }
  return classes[props.status || 'processing'] || 'text-gray-600'
})

// 格式化时间
function formatTime(seconds: number): string {
  if (seconds < 60) return `${seconds}秒`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}分钟`
  return `${Math.floor(seconds / 3600)}小时${Math.floor((seconds % 3600) / 60)}分钟`
}
</script>
