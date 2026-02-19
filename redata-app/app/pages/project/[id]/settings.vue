<template>
  <div class="space-y-6 h-full overflow-auto p-6">
    <!-- 基本信息 -->
    <div class="bg-elevated rounded-lg border border-default">
      <div class="px-6 py-4 border-b border-default flex items-center justify-between">
        <h3 class="text-base font-semibold text-highlighted">基本信息</h3>
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
            <label class="block text-sm font-medium text-default mb-2">
              项目名称
            </label>
            <div class="text-base text-highlighted">
              {{ project?.name || '-' }}
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-default mb-2">
              创建时间
            </label>
            <div class="text-base text-highlighted">
              {{ formatDateTime(project?.created_at) }}
            </div>
          </div>
        </div>
        <div>
          <label class="block text-sm font-medium text-default mb-2">
            项目描述
          </label>
          <div class="text-base text-highlighted whitespace-pre-wrap">
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
    <div class="bg-elevated rounded-lg border border-default">
      <div class="px-6 py-4 border-b border-default flex items-center justify-between">
        <h3 class="text-base font-semibold text-highlighted">数据统计</h3>
        <UButton
          icon="i-lucide-refresh-cw"
          size="sm"
          color="neutral"
          variant="ghost"
          :loading="loadingStats"
          @click="loadStatistics"
        >
          刷新
        </UButton>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-2 md:grid-cols-4 gap-6">
          <div class="text-center p-4 bg-muted rounded-lg">
            <div class="text-sm font-medium text-muted mb-2">总记录数</div>
            <div class="text-2xl font-bold text-highlighted">{{ statistics?.total_records ?? 0 }}</div>
          </div>
          <div class="text-center p-4 bg-muted rounded-lg">
            <div class="text-sm font-medium text-muted mb-2">今日新增</div>
            <div class="text-2xl font-bold text-primary">{{ statistics?.today_records ?? 0 }}</div>
          </div>
          <div class="text-center p-4 bg-muted rounded-lg">
            <div class="text-sm font-medium text-muted mb-2">本周新增</div>
            <div class="text-2xl font-bold text-primary">{{ statistics?.week_records ?? 0 }}</div>
          </div>
          <div class="text-center p-4 bg-muted rounded-lg">
            <div class="text-sm font-medium text-muted mb-2">最后处理</div>
            <div class="text-base font-semibold text-highlighted">
              {{ statistics?.last_processed_at ? formatDateTime(statistics.last_processed_at) : '-' }}
            </div>
          </div>
        </div>
        <div class="grid grid-cols-3 gap-6 mt-4 pt-4 border-t border-default">
          <div class="text-center p-3 bg-muted rounded-lg">
            <div class="text-sm font-medium text-muted mb-1">本月新增</div>
            <div class="text-xl font-bold text-highlighted">{{ statistics?.month_records ?? 0 }}</div>
          </div>
          <div class="text-center p-3 bg-muted rounded-lg">
            <div class="text-sm font-medium text-muted mb-1">处理任务数</div>
            <div class="text-xl font-bold text-highlighted">{{ statistics?.total_tasks ?? 0 }}</div>
          </div>
          <div class="text-center p-3 bg-muted rounded-lg">
            <div class="text-sm font-medium text-muted mb-1">成功率</div>
            <div class="text-xl font-bold text-success">{{ statistics?.success_rate?.toFixed(1) ?? 0 }}%</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 危险操作 -->
    <div class="bg-elevated rounded-lg border border-error/30">
      <div class="px-6 py-4 border-b border-error/30">
        <h3 class="text-base font-semibold text-error">危险操作</h3>
      </div>
      <div class="p-6">
        <div class="flex items-center justify-between p-4 bg-error/10 rounded-lg border border-error/30">
          <div>
            <div class="font-medium text-error">删除项目</div>
            <div class="text-sm text-error mt-1">
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
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProjectStore } from '~/stores/project'
import { statisticsApi, type ProjectStatistics } from '~/utils/api'

const route = useRoute()
const router = useRouter()
const toast = useToast()
const projectStore = useProjectStore()

const projectId = computed(() => Number(route.params.id))
const project = computed(() => projectStore.currentProject)

const isEditing = ref(false)
const saving = ref(false)

const editForm = reactive({ name: '', description: '' })

// 统计数据
const statistics = ref<ProjectStatistics | null>(null)
const loadingStats = ref(false)

// 监听项目变化,更新表单
watch(project, (newProject) => {
  if (newProject && !isEditing.value) {
    editForm.name = newProject.name
    editForm.description = newProject.description || ''
  }
}, { immediate: true })

onMounted(async () => {
  if (project.value) {
    editForm.name = project.value.name
    editForm.description = project.value.description || ''
  }
  await loadStatistics()
})

async function loadStatistics() {
  loadingStats.value = true
  try {
    statistics.value = await statisticsApi.get(projectId.value)
  } catch (error) {
    console.error('Failed to load statistics:', error)
  } finally {
    loadingStats.value = false
  }
}

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
