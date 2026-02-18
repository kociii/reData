<template>
  <div class="space-y-6">
    <!-- 基本信息 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <h3 class="text-base font-medium text-gray-900 dark:text-white">基本信息</h3>
      </div>
      <div class="p-4 space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <UFormField label="项目名称">
            <UInput v-model="editForm.name" />
          </UFormField>
          <UFormField label="创建时间">
            <UInput :model-value="formatDateTime(project?.created_at)" disabled />
          </UFormField>
        </div>
        <UFormField label="项目描述">
          <UTextarea v-model="editForm.description" :rows="2" />
        </UFormField>
        <div class="flex gap-2">
          <UButton :loading="saving" @click="saveBasicInfo">保存</UButton>
          <UButton color="neutral" variant="ghost" @click="resetForm">取消</UButton>
        </div>
      </div>
    </div>

    <!-- 数据统计 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <h3 class="text-base font-medium text-gray-900 dark:text-white">数据统计</h3>
      </div>
      <div class="p-4 grid grid-cols-4 gap-4">
        <div>
          <div class="text-sm text-gray-500 dark:text-gray-400">总记录数</div>
          <div class="text-xl font-bold text-gray-900 dark:text-white">0</div>
        </div>
        <div>
          <div class="text-sm text-gray-500 dark:text-gray-400">今日新增</div>
          <div class="text-xl font-bold text-gray-900 dark:text-white">0</div>
        </div>
        <div>
          <div class="text-sm text-gray-500 dark:text-gray-400">本周新增</div>
          <div class="text-xl font-bold text-gray-900 dark:text-white">0</div>
        </div>
        <div>
          <div class="text-sm text-gray-500 dark:text-gray-400">最后处理</div>
          <div class="text-xl font-bold text-gray-900 dark:text-white">-</div>
        </div>
      </div>
    </div>

    <!-- 危险操作 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-red-200 dark:border-red-900">
      <div class="px-4 py-3 border-b border-red-200 dark:border-red-900">
        <h3 class="text-base font-medium text-red-600 dark:text-red-400">危险操作</h3>
      </div>
      <div class="p-4 flex gap-3">
        <UButton color="neutral" variant="soft" @click="exportConfig">导出项目配置</UButton>
        <UButton color="warning" variant="soft" @click="clearData">清空所有数据</UButton>
        <UButton color="error" variant="soft" @click="deleteProject">删除项目</UButton>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProjectStore } from '~/stores/project'

const route = useRoute()
const router = useRouter()
const projectStore = useProjectStore()

const projectId = computed(() => Number(route.params.id))
const project = computed(() => projectStore.currentProject)

const saving = ref(false)

const editForm = reactive({ name: '', description: '' })

onMounted(() => {
  if (project.value) {
    editForm.name = project.value.name
    editForm.description = project.value.description || ''
  }
})

function resetForm() {
  if (project.value) {
    editForm.name = project.value.name
    editForm.description = project.value.description || ''
  }
}

async function saveBasicInfo() {
  saving.value = true
  try {
    await projectStore.updateProject(projectId.value, {
      name: editForm.name.trim(),
      description: editForm.description.trim() || undefined,
    })
  } catch (error) {
    console.error('Failed to save:', error)
  } finally {
    saving.value = false
  }
}

function exportConfig() {
  console.log('Export config for project:', projectId.value)
}

function clearData() {
  if (confirm('确定要清空所有数据吗？此操作无法撤销。')) {
    console.log('Clear data for project:', projectId.value)
  }
}

async function deleteProject() {
  if (confirm('确定要删除此项目吗？所有数据将被永久删除。')) {
    try {
      await projectStore.deleteProject(projectId.value)
      router.push('/')
    } catch (error) {
      console.error('Failed to delete project:', error)
    }
  }
}

function formatDateTime(dateStr?: string): string {
  if (!dateStr) return '-'
  return new Date(dateStr).toLocaleString('zh-CN')
}
</script>
