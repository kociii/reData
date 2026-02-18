<template>
  <div class="space-y-6">
    <!-- 基本信息 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <h3 class="text-base font-semibold text-gray-900 dark:text-white">基本信息</h3>
        <UButton
          v-if="!isEditing"
          icon="i-lucide-pencil"
          size="sm"
          color="neutral"
          variant="ghost"
          @click="startEdit"
        >
          编辑
        </UButton>
      </div>

      <!-- 只读态 -->
      <div v-if="!isEditing" class="p-6 space-y-6">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              项目名称
            </label>
            <div class="text-base text-gray-900 dark:text-white">
              {{ project?.name || '-' }}
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              创建时间
            </label>
            <div class="text-base text-gray-900 dark:text-white">
              {{ formatDateTime(project?.created_at) }}
            </div>
          </div>
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            项目描述
          </label>
          <div class="text-base text-gray-900 dark:text-white whitespace-pre-wrap">
            {{ project?.description || '暂无描述' }}
          </div>
        </div>
      </div>

      <!-- 编辑态 -->
      <div v-else class="p-6 space-y-6">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <UFormField label="项目名称" required class="w-full">
            <UInput
              v-model="editForm.name"
              size="lg"
              placeholder="请输入项目名称"
              class="w-full"
            />
          </UFormField>
          <UFormField label="创建时间" class="w-full">
            <UInput
              :model-value="formatDateTime(project?.created_at)"
              size="lg"
              disabled
              class="w-full"
            />
          </UFormField>
        </div>
        <UFormField label="项目描述" class="w-full">
          <UTextarea
            v-model="editForm.description"
            :rows="4"
            size="lg"
            placeholder="请输入项目描述（可选）"
            class="w-full"
          />
        </UFormField>
        <div class="flex gap-3 pt-2">
          <UButton
            :loading="saving"
            size="lg"
            @click="saveBasicInfo"
          >
            保存
          </UButton>
          <UButton
            color="neutral"
            variant="ghost"
            size="lg"
            @click="cancelEdit"
          >
            取消
          </UButton>
        </div>
      </div>
    </div>

    <!-- 数据统计 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
        <h3 class="text-base font-semibold text-gray-900 dark:text-white">数据统计</h3>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-2 md:grid-cols-4 gap-6">
          <div class="text-center p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
            <div class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2">总记录数</div>
            <div class="text-2xl font-bold text-gray-900 dark:text-white">0</div>
          </div>
          <div class="text-center p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
            <div class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2">今日新增</div>
            <div class="text-2xl font-bold text-primary-600 dark:text-primary-400">0</div>
          </div>
          <div class="text-center p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
            <div class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2">本周新增</div>
            <div class="text-2xl font-bold text-primary-600 dark:text-primary-400">0</div>
          </div>
          <div class="text-center p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
            <div class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2">最后处理</div>
            <div class="text-lg font-semibold text-gray-900 dark:text-white">-</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 危险操作 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-red-200 dark:border-red-900">
      <div class="px-6 py-4 border-b border-red-200 dark:border-red-900">
        <h3 class="text-base font-semibold text-red-600 dark:text-red-400">危险操作</h3>
      </div>
      <div class="p-6">
        <div class="space-y-4">
          <div class="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
            <div>
              <div class="font-medium text-gray-900 dark:text-white">导出项目配置</div>
              <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                导出项目的字段定义和配置信息
              </div>
            </div>
            <UButton
              color="neutral"
              variant="soft"
              size="lg"
              @click="exportConfig"
            >
              导出
            </UButton>
          </div>
          <div class="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
            <div>
              <div class="font-medium text-gray-900 dark:text-white">清空所有数据</div>
              <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                删除项目中的所有记录数据，但保留字段定义
              </div>
            </div>
            <UButton
              color="warning"
              variant="soft"
              size="lg"
              @click="clearData"
            >
              清空数据
            </UButton>
          </div>
          <div class="flex items-center justify-between p-4 bg-red-50 dark:bg-red-950 rounded-lg border border-red-200 dark:border-red-900">
            <div>
              <div class="font-medium text-red-600 dark:text-red-400">删除项目</div>
              <div class="text-sm text-red-500 dark:text-red-400 mt-1">
                永久删除此项目及所有相关数据，此操作无法撤销
              </div>
            </div>
            <UButton
              color="error"
              variant="soft"
              size="lg"
              @click="deleteProject"
            >
              删除项目
            </UButton>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProjectStore } from '~/stores/project'
import { useToast } from '#ui/composables/useToast'

const route = useRoute()
const router = useRouter()
const toast = useToast()
const projectStore = useProjectStore()

const projectId = computed(() => Number(route.params.id))
const project = computed(() => projectStore.currentProject)

const isEditing = ref(false)
const saving = ref(false)

const editForm = reactive({ name: '', description: '' })

// 监听项目变化,更新表单
watch(project, (newProject) => {
  if (newProject && !isEditing.value) {
    editForm.name = newProject.name
    editForm.description = newProject.description || ''
  }
}, { immediate: true })

onMounted(() => {
  if (project.value) {
    editForm.name = project.value.name
    editForm.description = project.value.description || ''
  }
})

function startEdit() {
  if (project.value) {
    editForm.name = project.value.name
    editForm.description = project.value.description || ''
  }
  isEditing.value = true
}

function cancelEdit() {
  if (project.value) {
    editForm.name = project.value.name
    editForm.description = project.value.description || ''
  }
  isEditing.value = false
}

async function saveBasicInfo() {
  if (!editForm.name.trim()) {
    toast.add({
      title: '项目名称不能为空',
      color: 'error',
    })
    return
  }

  saving.value = true
  try {
    await projectStore.updateProject(projectId.value, {
      name: editForm.name.trim(),
      description: editForm.description.trim() || undefined,
    })
    isEditing.value = false
    toast.add({
      title: '保存成功',
      color: 'success',
    })
  } catch (error) {
    console.error('Failed to save:', error)
    toast.add({
      title: '保存失败',
      description: error instanceof Error ? error.message : String(error),
      color: 'error',
    })
  } finally {
    saving.value = false
  }
}

function exportConfig() {
  console.log('Export config for project:', projectId.value)
  toast.add({
    title: '功能开发中',
    description: '导出配置功能即将上线',
    color: 'info',
  })
}

function clearData() {
  if (confirm('确定要清空所有数据吗？此操作无法撤销。')) {
    console.log('Clear data for project:', projectId.value)
    toast.add({
      title: '功能开发中',
      description: '清空数据功能即将上线',
      color: 'info',
    })
  }
}

async function deleteProject() {
  if (confirm('确定要删除此项目吗？所有数据将被永久删除，此操作无法撤销。')) {
    try {
      await projectStore.deleteProject(projectId.value)
      toast.add({
        title: '项目已删除',
        color: 'success',
      })
      router.push('/')
    } catch (error) {
      console.error('Failed to delete project:', error)
      toast.add({
        title: '删除失败',
        description: error instanceof Error ? error.message : String(error),
        color: 'error',
      })
    }
  }
}

function formatDateTime(dateStr?: string): string {
  if (!dateStr) return '-'
  return new Date(dateStr).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>
