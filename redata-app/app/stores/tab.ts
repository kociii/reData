import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { AppTab } from '~/types'

const HOME_TAB: AppTab = {
  id: 'home',
  label: '项目列表',
  path: '/',
  closable: false,
}

export const useTabStore = defineStore('tab', () => {
  // State
  const tabs = ref<AppTab[]>([{ ...HOME_TAB }])
  const activeTabId = ref('home')

  // Getters
  const activeTab = computed(() =>
    tabs.value.find(t => t.id === activeTabId.value) || tabs.value[0]
  )
  const projectTabs = computed(() =>
    tabs.value.filter(t => t.projectId != null)
  )

  // Actions
  function openProject(project: { id: number; name: string }) {
    const tabId = `project-${project.id}`
    const existing = tabs.value.find(t => t.id === tabId)
    if (existing) {
      activeTabId.value = tabId
      return existing
    }
    const tab: AppTab = {
      id: tabId,
      label: project.name,
      path: `/project/${project.id}/results`,
      closable: true,
      projectId: project.id,
    }
    tabs.value.push(tab)
    activeTabId.value = tabId
    return tab
  }

  function closeTab(id: string) {
    if (id === 'home') return
    const idx = tabs.value.findIndex(t => t.id === id)
    if (idx === -1) return
    tabs.value.splice(idx, 1)
    if (activeTabId.value === id) {
      // 切换到前一个标签或首页
      const newIdx = Math.min(idx, tabs.value.length - 1)
      activeTabId.value = tabs.value[newIdx]?.id || 'home'
    }
  }

  function setActiveTab(id: string) {
    if (tabs.value.find(t => t.id === id)) {
      activeTabId.value = id
    }
  }

  function updateTabPath(id: string, path: string) {
    const tab = tabs.value.find(t => t.id === id)
    if (tab) tab.path = path
  }

  function updateTabLabel(id: string, label: string) {
    const tab = tabs.value.find(t => t.id === id)
    if (tab) tab.label = label
  }

  function openSettings() {
    const tabId = 'settings'
    const existing = tabs.value.find(t => t.id === tabId)
    if (existing) {
      activeTabId.value = tabId
      return existing
    }
    const tab: AppTab = {
      id: tabId,
      label: '设置',
      path: '/settings',
      closable: true,
    }
    tabs.value.push(tab)
    activeTabId.value = tabId
    return tab
  }

  function removeProjectTab(projectId: number) {
    const tabId = `project-${projectId}`
    closeTab(tabId)
  }

  return {
    tabs,
    activeTabId,
    activeTab,
    projectTabs,
    openProject,
    openSettings,
    closeTab,
    setActiveTab,
    updateTabPath,
    updateTabLabel,
    removeProjectTab,
  }
})
