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
        <div class="space-y-4">
          <div class="flex items-center justify-between p-4 bg-muted rounded-lg">
            <div>
              <div class="font-medium text-highlighted">导出项目配置</div>
              <div class="text-sm text-muted mt-1">
                导出项目的字段定义和配置信息为 JSON 文件
              </div>
            </div>
            <UButton
              color="neutral"
              variant="soft"
              size="lg"
              :loading="exportingConfig"
              @click="exportConfig"
            >
              导出
            </UButton>
          </div>
          <div class="flex items-center justify-between p-4 bg-muted rounded-lg">
            <div>
              <div class="font-medium text-highlighted">导出项目数据</div>
              <div class="text-sm text-muted mt-1">
                导出所有记录数据为 Excel/CSV 文件
              </div>
            </div>
            <UButton
              color="neutral"
              variant="soft"
              size="lg"
              :loading="exportingData"
              @click="showExportDataModal = true"
            >
              导出
            </UButton>
          </div>
          <div class="flex items-center justify-between p-4 bg-muted rounded-lg">
            <div>
              <div class="font-medium text-highlighted">清空所有数据</div>
              <div class="text-sm text-muted mt-1">
                删除项目中的所有记录数据，但保留字段定义和项目配置
              </div>
            </div>
            <UButton
              color="warning"
              variant="soft"
              size="lg"
              :loading="clearingData"
              @click="showClearDataModal = true"
            >
              清空数据
            </UButton>
          </div>
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

    <!-- 清空数据确认弹窗 -->
    <UModal v-model:open="showClearDataModal">
      <template #content>
        <div class="p-6">
          <h3 class="text-lg font-semibold text-highlighted mb-4">确认清空数据</h3>
          <p class="text-default mb-4">
            此操作将删除项目 <strong class="text-highlighted">{{ project?.name }}</strong> 的所有记录数据（共 {{ statistics?.total_records ?? 0 }} 条），但会保留字段定义和项目配置。
          </p>
          <p class="text-error mb-4">
            请输入项目名称 "<strong>{{ project?.name }}</strong>" 以确认操作：
          </p>
          <UInput
            v-model="clearDataConfirmName"
            placeholder="请输入项目名称"
            class="mb-4"
          />
          <div class="flex gap-3 justify-end">
            <UButton
              color="neutral"
              variant="ghost"
              @click="showClearDataModal = false"
            >
              取消
            </UButton>
            <UButton
              color="warning"
              :disabled="clearDataConfirmName !== project?.name"
              :loading="clearingData"
              @click="confirmClearData"
            >
              确认清空
            </UButton>
          </div>
        </div>
      </template>
    </UModal>

    <!-- 导出数据弹窗 -->
    <UModal v-model:open="showExportDataModal">
      <template #content>
        <div class="p-6">
          <h3 class="text-lg font-semibold text-highlighted mb-4">导出数据</h3>
          <p class="text-default mb-4">
            选择导出格式：
          </p>
          <div class="space-y-3 mb-4">
            <label class="flex items-center gap-3 p-3 border rounded-lg cursor-pointer hover:bg-accented" :class="exportFormat === 'xlsx' ? 'border-primary bg-primary/5' : 'border-default'">
              <input type="radio" v-model="exportFormat" value="xlsx" class="text-primary" />
              <div>
                <div class="font-medium text-highlighted">Excel (.xlsx)</div>
                <div class="text-sm text-muted">推荐格式，支持多 Sheet</div>
              </div>
            </label>
            <label class="flex items-center gap-3 p-3 border rounded-lg cursor-pointer hover:bg-accented" :class="exportFormat === 'csv' ? 'border-primary bg-primary/5' : 'border-default'">
              <input type="radio" v-model="exportFormat" value="csv" class="text-primary" />
              <div>
                <div class="font-medium text-highlighted">CSV (.csv)</div>
                <div class="text-sm text-muted">通用格式，兼容性好</div>
              </div>
            </label>
          </div>
          <div class="flex gap-3 justify-end">
            <UButton
              color="neutral"
              variant="ghost"
              @click="showExportDataModal = false"
            >
              取消
            </UButton>
            <UButton
              color="primary"
              :loading="exportingData"
              @click="confirmExportData"
            >
              导出
            </UButton>
          </div>
        </div>
      </template>
    </UModal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProjectStore } from '~/stores/project'
import { useFieldStore } from '~/stores/field'
import { statisticsApi, type ProjectStatistics } from '~/utils/api'
import { recordsApi } from '~/utils/api'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile, writeBinaryFile } from '@tauri-apps/plugin-fs'

const route = useRoute()
const router = useRouter()
const toast = useToast()
const projectStore = useProjectStore()
const fieldStore = useFieldStore()

const projectId = computed(() => Number(route.params.id))
const project = computed(() => projectStore.currentProject)

const isEditing = ref(false)
const saving = ref(false)

const editForm = reactive({ name: '', description: '' })

// 统计数据
const statistics = ref<ProjectStatistics | null>(null)
const loadingStats = ref(false)

// 清空数据
const showClearDataModal = ref(false)
const clearDataConfirmName = ref('')
const clearingData = ref(false)

// 导出
const showExportDataModal = ref(false)
const exportFormat = ref<'xlsx' | 'csv'>('xlsx')
const exportingConfig = ref(false)
const exportingData = ref(false)

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

async function exportConfig() {
  exportingConfig.value = true
  try {
    // 获取字段定义
    const fields = await fieldStore.fetchFields(projectId.value)

    // 构建配置对象
    const config = {
      project: {
        name: project.value?.name,
        description: project.value?.description,
        dedup_enabled: project.value?.dedup_enabled,
        dedup_fields: project.value?.dedup_fields,
        dedup_strategy: project.value?.dedup_strategy,
      },
      fields: fields.map(f => ({
        field_name: f.field_name,
        field_label: f.field_label,
        field_type: f.field_type,
        is_required: f.is_required,
        is_dedup_key: f.is_dedup_key,
        additional_requirement: f.additional_requirement,
        validation_rule: f.validation_rule,
        extraction_hint: f.extraction_hint,
      })),
      exported_at: new Date().toISOString(),
      version: '1.0',
    }

    // 选择保存位置
    const filePath = await save({
      defaultPath: `${project.value?.name || 'project'}-config.json`,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })

    if (filePath) {
      await writeTextFile(filePath, JSON.stringify(config, null, 2))
      toast.add({
        title: '导出成功',
        description: `配置已保存到 ${filePath}`,
        color: 'success',
      })
    }
  } catch (error) {
    console.error('Failed to export config:', error)
    toast.add({
      title: '导出失败',
      description: error instanceof Error ? error.message : String(error),
      color: 'error',
    })
  } finally {
    exportingConfig.value = false
  }
}

async function confirmExportData() {
  exportingData.value = true
  try {
    // 获取所有记录
    const response = await recordsApi.query(projectId.value, { page: 1, pageSize: 10000 })
    const fields = fieldStore.fields

    if (response.records.length === 0) {
      toast.add({
        title: '没有数据',
        description: '项目中没有任何记录数据',
        color: 'warning',
      })
      return
    }

    // 选择保存位置
    const filePath = await save({
      defaultPath: `${project.value?.name || 'project'}-data.${exportFormat.value}`,
      filters: [{ name: exportFormat.value.toUpperCase(), extensions: [exportFormat.value] }],
    })

    if (filePath) {
      if (exportFormat.value === 'csv') {
        // CSV 格式
        const headers = ['ID', ...fields.map(f => f.field_label)]
        const rows = response.records.map((r: any) => [
          r.id,
          ...fields.map(f => r[f.id] ?? r.data?.[f.id] ?? '')
        ])
        const csv = [headers.join(','), ...rows.map(row => row.map(cell => `"${String(cell).replace(/"/g, '""')}"`).join(','))].join('\n')
        await writeTextFile(filePath, '\uFEFF' + csv) // 添加 BOM 以支持中文
      } else {
        // Excel 格式（简化为 CSV，实际应该使用 xlsx 库）
        const headers = ['ID', ...fields.map(f => f.field_label)]
        const rows = response.records.map((r: any) => [
          r.id,
          ...fields.map(f => r[f.id] ?? r.data?.[f.id] ?? '')
        ])
        const csv = [headers.join('\t'), ...rows.map(row => row.join('\t'))].join('\n')
        await writeTextFile(filePath, csv)
      }
      toast.add({
        title: '导出成功',
        description: `${response.records.length} 条记录已导出`,
        color: 'success',
      })
    }

    showExportDataModal.value = false
  } catch (error) {
    console.error('Failed to export data:', error)
    toast.add({
      title: '导出失败',
      description: error instanceof Error ? error.message : String(error),
      color: 'error',
    })
  } finally {
    exportingData.value = false
  }
}

async function confirmClearData() {
  clearingData.value = true
  try {
    const count = await recordsApi.deleteAll(projectId.value)
    toast.add({
      title: '数据已清空',
      description: `已删除 ${count} 条记录`,
      color: 'success',
    })
    showClearDataModal.value = false
    clearDataConfirmName.value = ''
    // 刷新统计
    await loadStatistics()
  } catch (error) {
    console.error('Failed to clear data:', error)
    toast.add({
      title: '清空失败',
      description: error instanceof Error ? error.message : String(error),
      color: 'error',
    })
  } finally {
    clearingData.value = false
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
