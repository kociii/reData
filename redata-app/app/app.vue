<template>
  <UApp>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </UApp>
</template>

<script setup lang="ts">
import { useProcessingStore } from '~/stores/processing'

// 全局初始化：启动事件监听器（不随页面卸载）
const processingStore = useProcessingStore()

onMounted(async () => {
  // 启动全局事件监听器
  await processingStore.startEventListener()
  console.log('[App] Global processing event listener started')
})
</script>
