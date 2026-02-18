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
      return 'bg-success border-success text-inverted'
    case 'active':
      return 'bg-primary border-primary text-inverted'
    case 'error':
      return 'bg-error border-error text-inverted'
    default:
      return 'bg-transparent border-default text-muted'
  }
}

function labelClass(status: ProcessingStage['status']) {
  switch (status) {
    case 'completed':
      return 'text-success font-medium'
    case 'active':
      return 'text-primary font-medium'
    case 'error':
      return 'text-error font-medium'
    default:
      return 'text-muted'
  }
}

function lineClass(status: ProcessingStage['status']) {
  return status === 'completed'
    ? 'bg-success'
    : 'bg-default'
}
</script>
