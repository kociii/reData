<template>
  <div class="h-full flex">
    <!-- 左侧分组列表 -->
    <div class="w-56 border-r border-default bg-elevated flex-shrink-0 flex flex-col">
      <div class="p-3 border-b border-default">
        <div class="flex items-center justify-between">
          <span class="text-sm font-medium">分组</span>
          <UButton
            icon="i-lucide-plus"
            color="neutral"
            variant="ghost"
            size="xs"
            @click="showCreateGroupModal = true"
          />
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-2">
        <!-- 全部 -->
        <div
          class="flex items-center justify-between px-2 py-1.5 rounded-md cursor-pointer mb-1"
          :class="selectedGroupId === null ? 'bg-primary/10 text-primary' : 'hover:bg-muted'"
          @click="selectGroup(null)"
        >
          <div class="flex items-center gap-2">
            <UIcon name="i-lucide-layout-grid" class="w-4 h-4" />
            <span class="text-sm">全部</span>
          </div>
          <span class="text-xs text-muted">{{ totalProjectCount }}</span>
        </div>

        <!-- 分组列表 -->
        <div class="space-y-0.5">
          <div
            v-for="group in groups"
            :key="group.id"
            class="rounded-md"
            :class="selectedGroupId === group.id ? 'bg-primary/10' : ''"
            :data-group-id="group.id"
          >
            <!-- 分组头部 -->
            <div
              class="flex items-center justify-between px-2 py-1.5 cursor-pointer group"
              :class="selectedGroupId === group.id ? '' : 'hover:bg-muted'"
              @click="selectGroup(group.id)"
            >
              <div class="flex items-center gap-2 min-w-0">
                <UIcon
                  v-if="group.children && group.children.length > 0"
                  :name="expandedGroups.has(group.id) ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
                  class="w-3 h-3 flex-shrink-0 text-muted"
                  @click.stop="toggleGroup(group.id)"
                />
                <div v-else class="w-3" />
                <UIcon
                  :name="group.icon || 'i-lucide-folder'"
                  class="w-4 h-4 flex-shrink-0"
                  :style="group.color ? { color: group.color } : {}"
                />
                <span class="text-sm truncate">{{ group.name }}</span>
              </div>
              <div class="flex items-center gap-1 flex-shrink-0">
                <span class="text-xs text-muted">{{ group.project_count }}</span>
                <UDropdownMenu :items="getGroupMenuItems(group)" :ui="{ content: 'w-32' }">
                  <UButton
                    icon="i-lucide-ellipsis"
                    color="neutral"
                    variant="ghost"
                    size="xs"
                    class="opacity-0 group-hover:opacity-100"
                    @click.stop
                  />
                </UDropdownMenu>
              </div>
            </div>

            <!-- 子分组 -->
            <div v-if="group.children && group.children.length > 0 && expandedGroups.has(group.id)" class="ml-4">
              <div
                v-for="child in group.children"
                :key="child.id"
                class="flex items-center justify-between px-2 py-1.5 cursor-pointer rounded-md"
                :class="selectedGroupId === child.id ? 'bg-primary/10 text-primary' : 'hover:bg-muted'"
                :data-group-id="child.id"
                @click="selectGroup(child.id)"
              >
                <div class="flex items-center gap-2 min-w-0">
                  <UIcon
                    :name="child.icon || 'i-lucide-folder'"
                    class="w-4 h-4 flex-shrink-0"
                    :style="child.color ? { color: child.color } : {}"
                  />
                  <span class="text-sm truncate">{{ child.name }}</span>
                </div>
                <div class="flex items-center gap-1 flex-shrink-0">
                  <span class="text-xs text-muted">{{ child.project_count }}</span>
                  <UDropdownMenu :items="getGroupMenuItems(child)" :ui="{ content: 'w-32' }">
                    <UButton
                      icon="i-lucide-ellipsis"
                      color="neutral"
                      variant="ghost"
                      size="xs"
                      @click.stop
                    />
                  </UDropdownMenu>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 右侧内容区 -->
    <div class="flex-1 flex flex-col min-h-0">
      <!-- 后端连接错误提示 -->
      <div
        v-if="backendError"
        class="bg-warning/10 border-b border-warning p-4"
      >
        <div class="flex items-center gap-2 text-warning">
          <UIcon name="i-lucide-triangle-alert" class="w-5 h-5" />
          <span class="font-medium">后端服务器未连接</span>
        </div>
        <p class="text-default text-sm mt-2">
          请先启动后端服务器：在 backend 目录下运行 <code class="bg-accented px-1 rounded">uv run python run.py</code>
        </p>
      </div>

      <!-- 页面标题 + 工具栏 -->
      <div class="flex justify-between items-center px-6 py-4 border-b border-default flex-shrink-0">
        <h2 class="text-lg font-semibold text-highlighted">
          {{ selectedGroupId ? getGroupName(selectedGroupId) : '全部项目' }}
        </h2>
        <UButton icon="i-lucide-plus" size="sm" @click="showCreateModal = true">
          新建项目
        </UButton>
      </div>

      <!-- 内容区 -->
      <div class="flex-1 overflow-auto p-6">
        <!-- 加载状态 -->
        <div v-if="projectStore.loading && !backendError" class="flex justify-center py-12">
          <div class="text-center">
            <UIcon name="i-lucide-refresh-cw" class="w-8 h-8 animate-spin text-primary mx-auto" />
            <p class="text-muted mt-2">加载项目中...</p>
          </div>
        </div>

        <!-- 空状态 -->
        <div
          v-else-if="filteredProjects.length === 0 && !backendError"
          class="text-center py-16 bg-elevated rounded-lg border border-default"
        >
          <UIcon name="i-lucide-folder-open" class="w-16 h-16 mx-auto text-dimmed mb-4" />
          <h3 class="text-lg font-medium text-highlighted mb-2">
            {{ selectedGroupId ? '该分组下暂无项目' : '还没有任何项目' }}
          </h3>
          <p class="text-sm text-muted mb-6">创建第一个项目开始使用 reData</p>
          <UButton icon="i-lucide-plus" @click="showCreateModal = true">新建项目</UButton>
        </div>

        <!-- 项目卡片列表 -->
        <div v-else class="grid grid-cols-[repeat(auto-fill,minmax(280px,1fr))] gap-4">
          <div
            v-for="project in filteredProjects"
            :key="project.id"
            class="bg-elevated rounded-lg border border-default hover:border-primary hover:shadow-md transition-all cursor-pointer"
            draggable="true"
            @click="openProject(project)"
            @dragstart="onDragStart($event, project)"
            @dragend="onDragEnd"
          >
            <div class="px-4 py-4">
              <div class="flex items-start justify-between gap-2">
                <div class="flex items-center gap-2 min-w-0">
                  <UIcon name="i-lucide-folder" class="w-5 h-5 text-primary flex-shrink-0" />
                  <h3 class="text-base font-medium text-highlighted truncate">{{ project.name }}</h3>
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
              <p class="text-sm text-muted line-clamp-2 mt-2">{{ project.description || '暂无描述' }}</p>
              <!-- 分组标签 -->
              <div v-if="project.group_id" class="mt-2">
                <UBadge
                  color="neutral"
                  variant="subtle"
                  size="xs"
                >
                  {{ getGroupName(project.group_id) }}
                </UBadge>
              </div>
            </div>
          </div>

          <!-- 新建项目卡片 -->
          <div
            class="border-2 border-dashed border-default rounded-lg flex items-center justify-center py-8 cursor-pointer hover:border-primary hover:bg-accented transition-colors"
            @click="showCreateModal = true"
          >
            <div class="flex items-center gap-2">
              <UIcon name="i-lucide-plus" class="w-5 h-5 text-muted" />
              <span class="text-sm text-muted">新建项目</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 拖拽提示覆盖层 -->
    <div
      v-if="isDragging"
      class="fixed inset-0 bg-primary/5 z-50 pointer-events-none flex items-center justify-center"
    >
      <div class="bg-elevated border-2 border-dashed border-primary rounded-lg px-6 py-4">
        <p class="text-primary">拖拽到左侧分组进行归类</p>
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
          <UFormField label="所属分组" name="group_id">
            <USelect
              v-model="form.group_id"
              :items="groupSelectItems"
              placeholder="选择分组（可选）"
            />
          </UFormField>
          <p class="text-sm text-muted">
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

    <!-- 删除项目确认对话框 -->
    <UModal v-model:open="showDeleteModal" title="确认删除">
      <template #body>
        <p class="text-default">确定要删除项目 "{{ projectToDelete?.name }}" 吗？此操作无法撤销，所有数据将被永久删除。</p>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" @click="showDeleteModal = false">取消</UButton>
          <UButton color="error" :loading="deleting" @click="deleteProject">删除</UButton>
        </div>
      </template>
    </UModal>

    <!-- 创建分组对话框 -->
    <UModal v-model:open="showCreateGroupModal" title="新建分组">
      <template #body>
        <UForm :state="groupForm" class="space-y-4" @submit="createGroup">
          <UFormField label="分组名称" name="name" required>
            <UInput v-model="groupForm.name" placeholder="输入分组名称" />
          </UFormField>
          <UFormField label="父分组" name="parent_id">
            <USelect
              v-model="groupForm.parent_id"
              :items="parentGroupSelectItems"
              placeholder="选择父分组（可选）"
            />
          </UFormField>
        </UForm>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" @click="showCreateGroupModal = false">取消</UButton>
          <UButton :loading="creatingGroup" :disabled="!groupForm.name.trim()" @click="createGroup">创建</UButton>
        </div>
      </template>
    </UModal>

    <!-- 编辑分组对话框 -->
    <UModal v-model:open="showEditGroupModal" title="编辑分组">
      <template #body>
        <UForm :state="editGroupForm" class="space-y-4" @submit="updateGroup">
          <UFormField label="分组名称" name="name" required>
            <UInput v-model="editGroupForm.name" placeholder="输入分组名称" />
          </UFormField>
        </UForm>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" @click="showEditGroupModal = false">取消</UButton>
          <UButton :loading="updatingGroup" :disabled="!editGroupForm.name.trim()" @click="updateGroup">保存</UButton>
        </div>
      </template>
    </UModal>

    <!-- 删除分组确认对话框 -->
    <UModal v-model:open="showDeleteGroupModal" title="确认删除分组">
      <template #body>
        <p class="text-default">
          确定要删除分组 "{{ groupToDelete?.name }}" 吗？
        </p>
        <p class="text-sm text-muted mt-2">
          该分组下的项目不会被删除，将移至"未分组"。
        </p>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" @click="showDeleteGroupModal = false">取消</UButton>
          <UButton color="error" :loading="deletingGroup" @click="deleteGroup">删除</UButton>
        </div>
      </template>
    </UModal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import type { DropdownMenuItem } from '@nuxt/ui'
import { useProjectStore } from '~/stores/project'
import { useTabStore } from '~/stores/tab'
import { projectGroupsApi, projectsApi } from '~/utils/api'
import type { Project, GroupWithChildren, ProjectGroupResponse } from '~/types'

definePageMeta({ layout: 'default' })

const projectStore = useProjectStore()
const tabStore = useTabStore()
const router = useRouter()
const toast = useToast()

// 后端状态
const backendError = ref(false)

// 分组相关
const groups = ref<GroupWithChildren[]>([])
const selectedGroupId = ref<number | null>(null)
const expandedGroups = ref<Set<number>>(new Set())
const loadingGroups = ref(false)

// 项目相关
const showCreateModal = ref(false)
const creating = ref(false)
const form = reactive({ name: '', description: '', group_id: null as number | null })
const showDeleteModal = ref(false)
const projectToDelete = ref<Project | null>(null)
const deleting = ref(false)

// 拖拽相关
const isDragging = ref(false)
const draggedProject = ref<Project | null>(null)

// 分组管理相关
const showCreateGroupModal = ref(false)
const creatingGroup = ref(false)
const groupForm = reactive({ name: '', parent_id: null as number | null })

const showEditGroupModal = ref(false)
const updatingGroup = ref(false)
const editGroupForm = reactive({ id: 0, name: '' })
const groupToEdit = ref<GroupWithChildren | null>(null)

const showDeleteGroupModal = ref(false)
const deletingGroup = ref(false)
const groupToDelete = ref<GroupWithChildren | null>(null)

// 计算属性
const totalProjectCount = computed(() => projectStore.projects.length)

const filteredProjects = computed(() => {
  if (selectedGroupId.value === null) {
    return projectStore.projects
  }
  return projectStore.projects.filter(p => p.group_id === selectedGroupId.value)
})

const groupSelectItems = computed(() => {
  const items: Array<{ label: string; value: number | null }> = [
    { label: '无分组', value: null }
  ]

  function addGroups(gs: GroupWithChildren[], prefix = '') {
    for (const g of gs) {
      items.push({ label: prefix + g.name, value: g.id })
      if (g.children && g.children.length > 0) {
        addGroups(g.children, prefix + '  ')
      }
    }
  }
  addGroups(groups.value)

  return items
})

const parentGroupSelectItems = computed(() => {
  const items: Array<{ label: string; value: number | null }> = [
    { label: '无（顶级分组）', value: null }
  ]

  function addGroups(gs: GroupWithChildren[]) {
    for (const g of gs) {
      items.push({ label: g.name, value: g.id })
      // 只允许两级，所以不添加子分组
    }
  }
  addGroups(groups.value)

  return items
})

// 加载数据
onMounted(async () => {
  try {
    await Promise.all([
      projectStore.fetchProjects(),
      loadGroups()
    ])
    backendError.value = false
  } catch (error) {
    console.error('Failed to fetch data:', error)
    backendError.value = true
  }
})

async function loadGroups() {
  loadingGroups.value = true
  try {
    groups.value = await projectGroupsApi.list()
  } catch (error) {
    console.error('Failed to load groups:', error)
  } finally {
    loadingGroups.value = false
  }
}

// 分组操作
function selectGroup(groupId: number | null) {
  selectedGroupId.value = groupId
}

function toggleGroup(groupId: number) {
  const newSet = new Set(expandedGroups.value)
  if (newSet.has(groupId)) {
    newSet.delete(groupId)
  } else {
    newSet.add(groupId)
  }
  expandedGroups.value = newSet
}

function getGroupName(groupId: number | null): string {
  if (groupId === null) return ''

  function findGroup(gs: GroupWithChildren[]): string {
    for (const g of gs) {
      if (g.id === groupId) return g.name
      if (g.children) {
        const found = findGroup(g.children)
        if (found) return found
      }
    }
    return ''
  }

  return findGroup(groups.value)
}

function getGroupMenuItems(group: GroupWithChildren): DropdownMenuItem[][] {
  return [
    [
      { label: '编辑', icon: 'i-lucide-pencil', onSelect: () => openEditGroup(group) },
    ],
    [
      { label: '删除', icon: 'i-lucide-trash-2', color: 'error' as const, onSelect: () => openDeleteGroup(group) },
    ],
  ]
}

function openEditGroup(group: GroupWithChildren) {
  groupToEdit.value = group
  editGroupForm.id = group.id
  editGroupForm.name = group.name
  showEditGroupModal.value = true
}

async function updateGroup() {
  if (!editGroupForm.name.trim()) return
  updatingGroup.value = true
  try {
    await projectGroupsApi.update(editGroupForm.id, { name: editGroupForm.name })
    await loadGroups()
    showEditGroupModal.value = false
    toast.add({ title: '分组已更新', color: 'success' })
  } catch (error: any) {
    toast.add({
      title: '更新失败',
      description: error?.message || String(error),
      color: 'error'
    })
  } finally {
    updatingGroup.value = false
  }
}

function openDeleteGroup(group: GroupWithChildren) {
  groupToDelete.value = group
  showDeleteGroupModal.value = true
}

async function deleteGroup() {
  if (!groupToDelete.value) return
  deletingGroup.value = true
  try {
    await projectGroupsApi.delete(groupToDelete.value.id)
    await Promise.all([loadGroups(), projectStore.fetchProjects()])
    if (selectedGroupId.value === groupToDelete.value.id) {
      selectedGroupId.value = null
    }
    showDeleteGroupModal.value = false
    toast.add({ title: '分组已删除', color: 'success' })
  } catch (error: any) {
    toast.add({
      title: '删除失败',
      description: error?.message || String(error),
      color: 'error'
    })
  } finally {
    deletingGroup.value = false
    groupToDelete.value = null
  }
}

async function createGroup() {
  if (!groupForm.name.trim()) return
  creatingGroup.value = true
  try {
    await projectGroupsApi.create({
      name: groupForm.name,
      parentId: groupForm.parent_id,
    })
    await loadGroups()
    showCreateGroupModal.value = false
    groupForm.name = ''
    groupForm.parent_id = null
    toast.add({ title: '分组已创建', color: 'success' })
  } catch (error: any) {
    toast.add({
      title: '创建失败',
      description: error?.message || String(error),
      color: 'error'
    })
  } finally {
    creatingGroup.value = false
  }
}

// 项目操作
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

    // 如果选择了分组，移动项目到该分组
    if (form.group_id) {
      await projectsApi.moveToGroup(project.id, form.group_id)
    }

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
  form.group_id = null
}

// 拖拽功能
function onDragStart(event: DragEvent, project: Project) {
  isDragging.value = true
  draggedProject.value = project
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', String(project.id))
  }
}

function onDragEnd() {
  isDragging.value = false
  draggedProject.value = null
}

// 监听拖拽到分组
onMounted(() => {
  const sidebar = document.querySelector('.w-56')
  if (sidebar) {
    sidebar.addEventListener('dragover', (e: Event) => {
      e.preventDefault()
      const de = e as DragEvent
      if (de.dataTransfer) {
        de.dataTransfer.dropEffect = 'move'
      }
    })

    sidebar.addEventListener('drop', async (e: Event) => {
      e.preventDefault()
      const de = e as DragEvent

      // 找到最近的分组元素
      const target = (de.target as HTMLElement).closest('[data-group-id]')
      if (target && draggedProject.value) {
        const groupId = parseInt(target.getAttribute('data-group-id') || '0')
        if (groupId) {
          try {
            await projectsApi.moveToGroup(draggedProject.value.id, groupId)
            await projectStore.fetchProjects()
            toast.add({ title: '项目已移动', color: 'success' })
          } catch (error: any) {
            toast.add({
              title: '移动失败',
              description: error?.message || String(error),
              color: 'error'
            })
          }
        }
      }
    })
  }
})
</script>
