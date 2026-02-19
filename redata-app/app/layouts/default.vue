<template>
  <div class="h-screen flex flex-col bg-muted">
    <!-- 顶部标题栏 -->
    <header class="bg-elevated shadow-sm border-b border-default flex-shrink-0">
      <div class="px-6 h-14 flex items-center justify-between">
        <!-- 左侧：Logo + 标题 -->
        <div class="flex items-center gap-2">
          <img src="/logo.svg" alt="reData Logo" class="w-8 h-8" />
          <span class="text-lg font-bold text-highlighted">reData</span>
        </div>

        <!-- 右侧：操作按钮 -->
        <div class="flex items-center gap-2">
          <UButton
            icon="i-lucide-settings"
            color="neutral"
            variant="ghost"
            size="sm"
            @click="openSettingsTab"
          />
          <UColorModeButton />
        </div>
      </div>
    </header>

    <!-- 全局标签栏 -->
    <nav class="bg-elevated border-b border-default flex-shrink-0">
      <div class="px-4 flex items-end gap-0.5 overflow-x-auto">
        <div
          v-for="tab in tabStore.tabs"
          :key="tab.id"
          class="group flex items-center gap-1.5 px-4 h-9 text-sm cursor-pointer border-b-2 transition-colors select-none flex-shrink-0"
          :class="tab.id === tabStore.activeTabId
            ? 'border-primary-500 text-primary bg-accented'
            : 'border-transparent text-muted hover:text-default hover:bg-accented'"
          @click="switchTab(tab)"
        >
          <UIcon
            :name="tab.id === 'settings' ? 'i-lucide-settings' : tab.projectId ? 'i-lucide-folder' : 'i-lucide-home'"
            class="w-4 h-4 flex-shrink-0"
          />
          <span class="truncate max-w-32">{{ tab.label }}</span>
          <button
            v-if="tab.closable"
            class="ml-1 p-0.5 rounded hover:bg-accented transition-colors"
            @click.stop="closeTab(tab.id)"
          >
            <UIcon name="i-lucide-x" class="w-3 h-3" />
          </button>
        </div>
      </div>
    </nav>

    <!-- 项目子导航栏（仅项目标签激活时显示） -->
    <nav v-if="isProjectPage" class="bg-elevated border-b border-default flex-shrink-0">
      <div class="px-6 flex gap-1">
        <NuxtLink
          :to="`/project/${projectId}/results`"
          class="px-4 py-2.5 text-sm font-medium transition-colors"
          :class="isResultsTab
            ? 'text-primary border-b-2 border-primary'
            : 'text-muted hover:text-default'"
        >
          数据结果
        </NuxtLink>
        <NuxtLink
          :to="`/project/${projectId}/processing`"
          class="px-4 py-2.5 text-sm font-medium transition-colors"
          :class="isProcessingTab
            ? 'text-primary border-b-2 border-primary'
            : 'text-muted hover:text-default'"
        >
          数据处理
        </NuxtLink>
        <NuxtLink
          :to="`/project/${projectId}/fields`"
          class="px-4 py-2.5 text-sm font-medium transition-colors"
          :class="isFieldsTab
            ? 'text-primary border-b-2 border-primary'
            : 'text-muted hover:text-default'"
        >
          字段定义
        </NuxtLink>
        <NuxtLink
          :to="`/project/${projectId}/settings`"
          class="px-4 py-2.5 text-sm font-medium transition-colors"
          :class="isSettingsTab
            ? 'text-primary border-b-2 border-primary'
            : 'text-muted hover:text-default'"
        >
          项目设置
        </NuxtLink>
      </div>
    </nav>

    <!-- 主内容区域 -->
    <main class="flex-1 overflow-hidden">
      <slot />
    </main>

    <!-- 底部状态栏 -->
    <footer class="bg-elevated border-t border-default flex-shrink-0">
      <div class="px-6 h-7 flex items-center justify-between text-xs text-muted">
        <div class="flex items-center gap-4">
          <template v-if="!isProjectPage">
            <span>共 {{ projectStore.projectCount }} 个项目</span>
            <span>版本: v1.0.0</span>
          </template>
          <template v-else-if="projectStore.currentProject">
            <span>项目: {{ projectStore.currentProject.name }}</span>
            <span>记录数: 0</span>
          </template>
        </div>
      </div>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, watch } from 'vue'
import { useProjectStore } from '~/stores/project'
import { useTabStore } from '~/stores/tab'
import type { AppTab } from '~/types'

const projectStore = useProjectStore()
const tabStore = useTabStore()
const router = useRouter()
const route = useRoute()

// 计算属性
const isProjectPage = computed(() => route.path.startsWith('/project/'))
const projectId = computed(() => {
  const id = route.params.id
  return Array.isArray(id) ? id[0] : id
})

// 项目子导航 Tab 判断
const isSettingsTab = computed(() => route.path.includes('/settings') && isProjectPage.value)
const isFieldsTab = computed(() => route.path.includes('/fields'))
const isProcessingTab = computed(() => route.path.includes('/processing'))
const isResultsTab = computed(() =>
  route.path.includes('/results') ||
  (isProjectPage.value && !isSettingsTab.value && !isFieldsTab.value && !isProcessingTab.value)
)

// 切换标签
function switchTab(tab: AppTab) {
  tabStore.setActiveTab(tab.id)
  router.push(tab.path)
}

// 打开设置标签页
function openSettingsTab() {
  tabStore.openSettings()
  router.push('/settings')
}

// 关闭标签
function closeTab(id: string) {
  const wasActive = tabStore.activeTabId === id
  tabStore.closeTab(id)
  // 仅当关闭的是当前激活标签时才导航
  if (wasActive) {
    router.push(tabStore.activeTab.path)
  }
}

// 监听路由变化，同步标签状态
watch(
  () => route.fullPath,
  (newPath) => {
    if (newPath.startsWith('/project/')) {
      const id = route.params.id
      if (id) {
        const tabId = `project-${id}`
        // 如果标签存在，更新路径和激活状态
        if (tabStore.tabs.find(t => t.id === tabId)) {
          tabStore.setActiveTab(tabId)
          tabStore.updateTabPath(tabId, newPath)
        }
      }
    } else if (newPath === '/settings') {
      // 设置页激活 settings 标签
      if (tabStore.tabs.find(t => t.id === 'settings')) {
        tabStore.setActiveTab('settings')
      }
    } else {
      tabStore.setActiveTab('home')
    }
  }
)

// 监听路由变化，更新当前项目
watch(
  () => route.params.id,
  (newId) => {
    if (newId && projectStore.projects.length > 0) {
      const project = projectStore.projects.find((p) => p.id === Number(newId))
      if (project) {
        projectStore.setCurrentProject(project)
      }
    }
  }
)

// 初始化
onMounted(async () => {
  await projectStore.fetchProjects()

  const id = route.params.id
  if (id) {
    const numId = Number(id)
    const project = projectStore.projects.find((p) => p.id === numId)
    if (project) {
      projectStore.setCurrentProject(project)
    }
  }
})
</script>
