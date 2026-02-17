<script setup lang="ts">
const route = useRoute()
const router = useRouter()

// 默认重定向到数据结果页
onMounted(() => {
  router.replace(`/project/${route.params.id}/results`)
})
</script>

<template>
  <div class="flex justify-center py-12">
    <UIcon name="i-lucide-refresh-cw" class="w-8 h-8 animate-spin text-primary" />
  </div>
</template>
