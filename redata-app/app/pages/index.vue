<template>
  <div class="space-y-4">
    <!-- 后端连接错误提示 -->
    <div
      v-if="backendError"
      class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4"
    >
      <div class="flex items-center gap-2 text-yellow-800 dark:text-yellow-200">
        <UIcon name="i-lucide-triangle-alert" class="w-5 h-5" />
        <span class="font-medium">后端服务器未连接</span>
      </div>
      <p class="text-yellow-700 dark:text-yellow-300 text-sm mt-2">
        请先启动后端服务器：在 backend 目录下运行 <code class="bg-yellow-100 dark:bg-yellow-900 px-1 rounded">uv run python run.py</code>
      </p>
    </div>

    <!-- 页面标题 + 工具栏 -->
    <div class="flex justify-between items-center">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white">我的项目</h2>
      <UButton icon="i-lucide-plus" size="sm" @click="showCreateModal = true">
        新建项目
      </UButton>
    </div>

    <!-- 加载状态 -->
    <div v-if="projectStore.loading && !backendError" class="flex justify-center py-12">
      <div class="text-center">
        <UIcon name="i-lucide-refresh-cw" class="w-8 h-8 animate-spin text-primary mx-auto" />
        <p class="text-gray-500 dark:text-gray-400 mt-2">加载项目中...</p>
      </div>
    </div>

    <!-- 空状态 -->
    <div
      v-else-if="!projectStore.hasProjects && !backendError"
      class="text-center py-16 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700"
    >
      <UIcon name="i-lucide-folder-open" class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
      <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">还没有任何项目</h3>
      <p class="text-sm text-gray-500 dark:text-gray-400 mb-6">创建第一个项目开始使用 reData</p>
      <UButton icon="i-lucide-plus" @click="showCreateModal = true">新建项目</UButton>
    </div>

    <!-- 项目卡片列表（弹性布局） -->
    <div v-else-if="projectStore.hasProjects" class="grid grid-cols-[repeat(auto-fill,minmax(280px,1fr))] gap-4">
      <div
        v-for="project in projectStore.projects"
        :key="project.id"
        class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-primary-300 dark:hover:border-primary-600 hover:shadow-md transition-all cursor-pointer"
        @click="openProject(project)"
      >
        <div class="px-4 py-4">
          <div class="flex items-start justify-between gap-2">
            <div class="flex items-center gap-2 min-w-0">
              <UIcon name="i-lucide-folder" class="w-5 h-5 text-primary-500 flex-shrink-0" />
              <h3 class="text-base font-medium text-gray-900 dark:text-white truncate">{{ project.name }}</h3>
            </div>
            <UDropdownMenu :items="getProjectMenuItems(project)" :ui="{ content: 'w-36' }">
              <UButton
                icon="i-lucide-ellipsis"
                color="neutral"
                variant="ghost"
                size="xs"
                @click.stop
              />
            </UDropdownMenu>
          </div>
          <p class="text-sm text-gray-500 dark:text-gray-400 line-clamp-2 mt-2">{{ project.description || '暂无描述' }}</p>
        </div>
      </div>

      <!-- 新建项目卡片 -->
      <div
        class="border-2 border-dashed border-gray-200 dark:border-gray-700 rounded-lg flex items-center justify-center py-8 cursor-pointer hover:border-primary-300 dark:hover:border-primary-600 hover:bg-primary-50/50 dark:hover:bg-primary-900/10 transition-colors"
        @click="showCreateModal = true"
      >
        <div class="flex items-center gap-2">
          <UIcon name="i-lucide-plus" class="w-5 h-5 text-gray-400" />
          <span class="text-sm text-gray-500 dark:text-gray-400">新建项目</span>
        </div>
      </div>
    </div>

    <!-- 创建项目对话框 -->
    <UModal v-model:open="showCreateModal" title="创建新项目">
      <template #body>
        <UForm :state="form" class="space-y-4" @submit="createProject">
          <UFormField label="项目名称" name="name" required>
            <UInput v-model="form.name" placeholder="输入项目名称" />
          </UFormField>
          <UFormField label="项目描述" name="description">
            <UTextarea v-model="form.description" placeholder="可选的项目描述" :rows="3" />
          </UFormField>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            去重配置可在字段定义中设置（每个字段可单独设置是否参与去重）
          </p>
        </UForm>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" @click="showCreateModal = false">取消</UButton>
          <UButton :loading="creating" :disabled="!form.name.trim()" @click="createProject">创建</UButton>
        </div>
      </template>
    </UModal>

    <!-- 删除确认对话框 -->
    <UModal v-model:open="showDeleteModal" title="确认删除">
      <template #body>
        <p class="text-gray-600 dark:text-gray-400">确定要删除项目 "{{ projectToDelete?.name }}" 吗？此操作无法撤销，所有数据将被永久删除。</p>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" @click="showDeleteModal = false">取消</UButton>
          <UButton color="error" :loading="deleting" @click="deleteProject">删除</UButton>
        </div>
      </template>
    </UModal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import type { DropdownMenuItem } from '@nuxt/ui'
import { useProjectStore } from '~/stores/project'
import { useTabStore } from '~/stores/tab'
import type { Project } from '~/types'

definePageMeta({ layout: 'default' })

const projectStore = useProjectStore()
const tabStore = useTabStore()
const router = useRouter()

const backendError = ref(false)
const showCreateModal = ref(false)
const creating = ref(false)
const form = reactive({ name: '', description: '' })
const showDeleteModal = ref(false)
const projectToDelete = ref<Project | null>(null)
const deleting = ref(false)

onMounted(async () => {
  try {
    await projectStore.fetchProjects()
    backendError.value = false
  } catch (error) {
    console.error('Failed to fetch projects:', error)
    backendError.value = true
  }
})

function getProjectMenuItems(project: Project): DropdownMenuItem[][] {
  return [
    [
      { label: '字段定义', icon: 'i-lucide-square-pen', onSelect: () => openFields(project) },
      { label: '项目设置', icon: 'i-lucide-settings', onSelect: () => editProject(project) },
      { label: '导出数据', icon: 'i-lucide-download', onSelect: () => exportProject(project) },
    ],
    [
      { label: '删除项目', icon: 'i-lucide-trash-2', color: 'error' as const, onSelect: () => confirmDelete(project) },
    ],
  ]
}

function openProject(project: Project) {
  projectStore.setCurrentProject(project)
  const tab = tabStore.openProject(project)
  router.push(tab.path)
}

function openFields(project: Project) {
  projectStore.setCurrentProject(project)
  const tab = tabStore.openProject(project)
  const fieldsPath = `/project/${project.id}/fields`
  tabStore.updateTabPath(tab.id, fieldsPath)
  router.push(fieldsPath)
}

function editProject(project: Project) {
  projectStore.setCurrentProject(project)
  const tab = tabStore.openProject(project)
  const settingsPath = `/project/${project.id}/settings`
  tabStore.updateTabPath(tab.id, settingsPath)
  router.push(settingsPath)
}

function exportProject(project: Project) {
  console.log('Export project:', project.id)
}

function confirmDelete(project: Project) {
  projectToDelete.value = project
  showDeleteModal.value = true
}

async function createProject() {
  if (!form.name.trim()) return
  creating.value = true
  try {
    const project = await projectStore.createProject({
      name: form.name.trim(),
      description: form.description.trim() || undefined,
    })
    showCreateModal.value = false
    resetForm()
    await projectStore.fetchProjects()
    const tab = tabStore.openProject(project)
    router.push(tab.path)
  } catch (error) {
    console.error('Failed to create project:', error)
    alert('创建项目失败，请检查后端服务器是否运行')
  } finally {
    creating.value = false
  }
}

async function deleteProject() {
  if (!projectToDelete.value) return
  deleting.value = true
  try {
    await projectStore.deleteProject(projectToDelete.value.id)
    tabStore.removeProjectTab(projectToDelete.value.id)
    showDeleteModal.value = false
    projectToDelete.value = null
  } catch (error) {
    console.error('Failed to delete project:', error)
  } finally {
    deleting.value = false
  }
}

function resetForm() {
  form.name = ''
  form.description = ''
}

function formatDate(dateStr: string): string {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const hours = Math.floor(diff / (1000 * 60 * 60))
  if (hours < 1) return '刚刚'
  if (hours < 24) return `${hours}小时前`
  const days = Math.floor(hours / 24)
  if (days < 7) return `${days}天前`
  if (days < 30) return `${Math.floor(days / 7)}周前`
  return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
}
</script>