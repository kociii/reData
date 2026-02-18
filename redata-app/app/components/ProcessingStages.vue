<template>
  <div class="flex items-center gap-0">
    <template v-for="(stage, index) in stages" :key="stage.key">
      <!-- 节点 -->
      <div class="flex flex-col items-center gap-1.5 min-w-0">
        <div
          class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-medium border-2 transition-all"
          :class="nodeClass(stage.status)"
        >
          <UIcon v-if="stage.status === 'completed'" name="i-lucide-check" class="w-3.5 h-3.5" />
          <UIcon v-else-if="stage.status === 'error'" name="i-lucide-x" class="w-3.5 h-3.5" />
          <UIcon v-else-if="stage.status === 'active'" name="i-lucide-loader" class="w-3.5 h-3.5 animate-spin" />
          <span v-else>{{ index + 1 }}</span>
        </div>
        <span
          class="text-xs whitespace-nowrap"
          :class="labelClass(stage.status)"
        >
          {{ stage.label }}
        </span>
      </div>
      <!-- 连接线 -->
      <div
        v-if="index < stages.length - 1"
        class="flex-1 h-0.5 min-w-6 mb-5 transition-all"
        :class="lineClass(stage.status)"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import type { ProcessingStage } from '~/types'

defineProps<{
  stages: ProcessingStage[]
}>()

function nodeClass(status: ProcessingStage['status']) {
  switch (status) {
    case 'completed':
      return 'bg-green-500 border-green-500 text-white'
    case 'active':
      return 'bg-primary border-primary text-white'
    case 'error':
      return 'bg-red-500 border-red-500 text-white'
    default:
      return 'bg-transparent border-gray-300 dark:border-gray-600 text-gray-400 dark:text-gray-500'
  }
}

function labelClass(status: ProcessingStage['status']) {
  switch (status) {
    case 'completed':
      return 'text-green-600 dark:text-green-400 font-medium'
    case 'active':
      return 'text-primary font-medium'
    case 'error':
      return 'text-red-600 dark:text-red-400 font-medium'
    default:
      return 'text-gray-400 dark:text-gray-500'
  }
}

function lineClass(status: ProcessingStage['status']) {
  return status === 'completed'
    ? 'bg-green-500'
    : 'bg-gray-300 dark:bg-gray-600'
}
</script>
