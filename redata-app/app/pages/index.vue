<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
    <div class="text-center">
      <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">
        reData
      </h1>
      <p class="text-xl text-gray-600 dark:text-gray-400 mb-8">
        智能数据处理平台
      </p>
      <UButton size="lg" @click="testGreet">
        测试 Tauri 连接
      </UButton>
      <p v-if="greetMsg" class="mt-4 text-gray-700 dark:text-gray-300">
        {{ greetMsg }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const greetMsg = ref('')

async function testGreet() {
  try {
    // @ts-ignore
    const { invoke } = await import('@tauri-apps/api/core')
    greetMsg.value = await invoke('greet', { name: 'reData' })
  } catch (error) {
    greetMsg.value = '开发模式：Tauri 未连接'
    console.log('Running in dev mode without Tauri')
  }
}
</script>
