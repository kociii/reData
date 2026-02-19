<template>
  <div class="h-full">
    <!-- 加载状态 -->
    <div v-if="loading" class="flex justify-center py-12">
      <UIcon name="i-lucide-refresh-cw" class="w-8 h-8 animate-spin text-primary" />
    </div>

    <!-- 项目不存在 -->
    <div v-else-if="!project" class="text-center py-16">
      <UIcon name="i-lucide-triangle-alert" class="w-12 h-12 mx-auto text-yellow-500 mb-3" />
      <h3 class="text-base font-medium text-gray-900 dark:text-white mb-1">项目不存在</h3>
      <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">该项目可能已被删除</p>
      <UButton size="sm" to="/">返回首页</UButton>
    </div>

    <!-- 子页面渲染 -->
    <NuxtPage v-else />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useProjectStore } from '~/stores/project'
import { useFieldStore } from '~/stores/field'
import { useTabStore } from '~/stores/tab'

definePageMeta({
  layout: 'default',
})

const route = useRoute()
const router = useRouter()
const projectStore = useProjectStore()
const fieldStore = useFieldStore()
const tabStore = useTabStore()

const projectId = computed(() => Number(route.params.id))
const project = computed(() => projectStore.currentProject)
const loading = ref(true)

// 加载项目数据
async function loadProject() {
  loading.value = true
  try {
    await projectStore.fetchProjects()

    const projectFromList = projectStore.projects.find(p => p.id === projectId.value)
    if (projectFromList) {
      projectStore.setCurrentProject(projectFromList)
      // 确保有对应的标签（直接通过 URL 访问时）
      tabStore.openProject(projectFromList)
      tabStore.updateTabPath(`project-${projectFromList.id}`, route.fullPath)
    } else {
      await projectStore.fetchProject(projectId.value)
      if (projectStore.currentProject) {
        tabStore.openProject(projectStore.currentProject)
        tabStore.updateTabPath(`project-${projectStore.currentProject.id}`, route.fullPath)
      }
    }

    await fieldStore.fetchFields(projectId.value)
  } catch (error) {
    console.error('Failed to load project:', error)
    router.push('/')
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadProject()
})
</script>
