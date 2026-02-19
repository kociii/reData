import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import type {
  Project,
  CreateProjectRequest,
  UpdateProjectRequest,
  ProjectField,
  CreateFieldRequest,
  GenerateFieldMetadataRequest,
  FieldMetadata,
  AiConfig,
  CreateAiConfigRequest,
  ProcessingTask,
  StartProcessingRequest,
  QueryResult,
  ExcelPreview,
  FieldDefinition,
  ColumnMappingResponse,
  ProjectRecord,
  QueryRecordsResponse,
  Batch,
  ListTasksResponse,
  ProcessingProgress,
  FullTaskProgressResponse,
  RollbackResult,
  BatchDetailResponse,
} from '~/types'

// 使用 Tauri Commands 模式（零网络开销）
const USE_TAURI_COMMANDS = true

console.log(`Using ${USE_TAURI_COMMANDS ? 'Tauri Commands' : 'HTTP API'} backend`)

// 通用请求函数
async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const url = `${API_BASE}${path}`
  const response = await fetch(url, {
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
    ...options,
  })

  if (!response.ok) {
    const error = await response.text()
    throw new Error(`API Error: ${response.status} - ${error}`)
  }

  return response.json()
}

// ============ 项目 API ============

export const projectsApi = {
  list: async () => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<Project[]>('get_projects')
    }
    return request<Project[]>('/projects')
  },

  get: async (id: number) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<Project>('get_project', { id })
    }
    return request<Project>(`/projects/${id}`)
  },

  create: async (data: CreateProjectRequest) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<Project>('create_project', {
        name: data.name,
        description: data.description,
      })
    }
    return request<Project>('/projects', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  },

  update: async (id: number, data: UpdateProjectRequest) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<Project>('update_project', {
        id,
        name: data.name,
        description: data.description,
        dedup_enabled: data.dedup_enabled,
        dedup_fields: data.dedup_fields,
        dedup_strategy: data.dedup_strategy,
      })
    }
    return request<Project>(`/projects/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  },

  delete: async (id: number) => {
    if (USE_TAURI_COMMANDS) {
      await invoke<void>('delete_project', { id })
      return
    }
    return request<void>(`/projects/${id}`, {
      method: 'DELETE',
    })
  },
}

// ============ 字段 API ============

export const fieldsApi = {
  list: async (projectId: number) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<ProjectField[]>('get_fields', { projectId })
    }
    return request<ProjectField[]>(`/fields/project/${projectId}`)
  },

  listAll: async (projectId: number) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<ProjectField[]>('get_all_fields', { projectId })
    }
    // HTTP API 不支持此功能，返回普通列表
    return request<ProjectField[]>(`/fields/project/${projectId}`)
  },

  create: async (data: CreateFieldRequest) => {
    if (USE_TAURI_COMMANDS) {
      console.log('[fieldsApi.create] Input data:', JSON.stringify(data))

      // 参数验证
      const projectId = Number(data.project_id)
      if (!projectId || isNaN(projectId) || projectId <= 0) {
        const error = `无效的项目 ID: ${data.project_id}`
        console.error('[fieldsApi.create] Validation error:', error)
        throw new Error(error)
      }

      const params = {
        projectId,
        fieldName: String(data.field_name || ''),
        fieldLabel: String(data.field_label || ''),
        fieldType: String(data.field_type || 'text'),
        isRequired: data.is_required ?? false,
        isDedupKey: data.is_dedup_key ?? false,
        additionalRequirement: data.additional_requirement || null,
        validationRule: data.validation_rule || null,
        extractionHint: data.extraction_hint || null,
      }
      console.log('[fieldsApi.create] Prepared params:', JSON.stringify(params))
      console.log('[fieldsApi.create] About to call invoke...')

      try {
        const result = await invoke<ProjectField>('create_field', params)
        console.log('[fieldsApi.create] Success:', result)
        return result
      } catch (error) {
        console.error('[fieldsApi.create] Error:', error)
        throw error
      }
    }
    return request<ProjectField>('/fields/', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  },

  update: async (id: number, data: Partial<CreateFieldRequest>) => {
    if (USE_TAURI_COMMANDS) {
      // 前端参数验证
      if (!id || id <= 0) {
        throw new Error(`无效的字段 ID: ${id}（ID 必须为正整数）`)
      }

      const params = {
        id: Number(id),
        fieldName: data.field_name || null,
        fieldLabel: data.field_label || null,
        fieldType: data.field_type || null,
        isRequired: data.is_required ?? null,
        isDedupKey: data.is_dedup_key ?? null,
        additionalRequirement: data.additional_requirement || null,
        validationRule: data.validation_rule || null,
        extractionHint: data.extraction_hint || null,
      }
      console.log('[fieldsApi.update] Calling with params:', JSON.stringify(params))
      try {
        const result = await invoke<ProjectField>('update_field', params)
        console.log('[fieldsApi.update] Success:', result)
        return result
      } catch (error) {
        console.error('[fieldsApi.update] Error:', error)
        // 确保错误是 Error 对象
        const err = error instanceof Error ? error : new Error(String(error))
        throw err
      }
    }
    return request<ProjectField>(`/fields/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  },

  delete: async (id: number) => {
    if (USE_TAURI_COMMANDS) {
      await invoke<void>('delete_field', { id })
      return
    }
    return request<void>(`/fields/${id}`, {
      method: 'DELETE',
    })
  },

  restore: async (id: number) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<ProjectField>('restore_field', { id })
    }
    // HTTP API 不支持此功能
    throw new Error('restore_field not supported in HTTP API mode')
  },

  generateMetadata: async (data: GenerateFieldMetadataRequest) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<FieldMetadata>('generate_field_metadata', {
        fieldLabel: data.field_label,
        fieldType: data.field_type,
        additionalRequirement: data.additional_requirement,
      })
    }
    return request<FieldMetadata>('/fields/generate-metadata', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  },
}

// ============ AI 配置 API ============

export const aiConfigsApi = {
  list: async () => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<AiConfig[]>('get_ai_configs')
    }
    return request<AiConfig[]>('/ai-configs/')
  },

  get: async (id: number) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<AiConfig>('get_ai_config', { id })
    }
    return request<AiConfig>(`/ai-configs/${id}`)
  },

  getDefault: async () => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<AiConfig>('get_default_ai_config')
    }
    return request<AiConfig>('/ai-configs/default')
  },

  create: async (data: CreateAiConfigRequest) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<AiConfig>('create_ai_config', {
        name: data.name,
        apiUrl: data.api_url,
        modelName: data.model_name,
        apiKey: data.api_key,
        temperature: data.temperature,
        maxTokens: data.max_tokens,
        isDefault: data.is_default,
      })
    }
    return request<AiConfig>('/ai-configs/', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  },

  update: async (id: number, data: Partial<CreateAiConfigRequest>) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<AiConfig>('update_ai_config', {
        id,
        name: data.name,
        apiUrl: data.api_url,
        modelName: data.model_name,
        apiKey: data.api_key,
        temperature: data.temperature,
        maxTokens: data.max_tokens,
        isDefault: data.is_default,
      })
    }
    return request<AiConfig>(`/ai-configs/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  },

  delete: async (id: number) => {
    if (USE_TAURI_COMMANDS) {
      await invoke<void>('delete_ai_config', { id })
      return
    }
    return request<void>(`/ai-configs/${id}`, {
      method: 'DELETE',
    })
  },

  testConnection: async (id: number) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<{ success: boolean; message: string; response: string | null }>('test_ai_connection', { id })
    }
    return request<{ success: boolean; message: string; response: string }>(`/ai-configs/test-connection`, {
      method: 'POST',
      body: JSON.stringify({ config_id: id }),
    })
  },

  setDefault: async (id: number) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<AiConfig>('set_default_ai_config', { id })
    }
    return request<AiConfig>(`/ai-configs/${id}/set-default`, {
      method: 'POST',
    })
  },
}

// ============ 文件 API ============

export const filesApi = {
  getSheets: async (filePath: string) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<{ name: string; row_count: number; column_count: number }[]>('get_excel_sheets', { filePath })
    }
    // HTTP API 回退
    const formData = new FormData()
    formData.append('file', await fetch(filePath).then(r => r.blob()))
    return request<{ name: string; row_count: number; column_count: number }[]>('/files/sheets', {
      method: 'POST',
      body: formData,
    })
  },

  preview: async (filePath: string, sheetName?: string, maxRows?: number): Promise<ExcelPreview> => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<ExcelPreview>('preview_excel', { filePath, sheetName, maxRows })
    }
    const formData = new FormData()
    formData.append('file', await fetch(filePath).then(r => r.blob()))
    const params = new URLSearchParams()
    if (sheetName) params.append('sheet_name', sheetName)
    if (maxRows) params.append('max_rows', maxRows.toString())
    const response = await fetch(`${API_BASE}/files/preview?${params}`, {
      method: 'POST',
      body: formData,
    })
    if (!response.ok) {
      throw new Error('Failed to preview file')
    }
    return response.json()
  },
}

// ============ 处理 API ============

export const processingApi = {
  start: async (data: StartProcessingRequest) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<{ task_id: string; batch_number: string; project_id: number; status: string; source_files: string[] }>('start_processing', {
        projectId: data.project_id,
        filePaths: data.file_paths,
        aiConfigId: data.ai_config_id,
      })
    }
    return request<ProcessingTask>('/processing/start', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  },

  pause: async (taskId: string) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<void>('pause_processing_task', { taskId })
    }
    return request<ProcessingTask>(`/processing/pause/${taskId}`, {
      method: 'POST',
    })
  },

  resume: async (taskId: string) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<void>('resume_processing_task', { taskId })
    }
    return request<ProcessingTask>(`/processing/resume/${taskId}`, {
      method: 'POST',
    })
  },

  cancel: async (taskId: string) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<void>('cancel_processing_task', { taskId })
    }
    return request<ProcessingTask>(`/processing/cancel/${taskId}`, {
      method: 'POST',
    })
  },

  status: async (taskId: string) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<ProcessingTask>('get_processing_task', { taskId })
    }
    return request<ProcessingTask>(`/processing/status/${taskId}`)
  },

  list: async (projectId: number, status?: string) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<ListTasksResponse>('list_processing_tasks', { projectId, status })
    }
    const params = new URLSearchParams()
    if (status) params.append('status', status)
    return request<{ tasks: ProcessingTask[]; total: number }>(
      `/processing/list/${projectId}?${params.toString()}`
    )
  },

  // 获取任务完整进度（文件和 Sheet 级别）
  getFullProgress: async (taskId: string): Promise<FullTaskProgressResponse> => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<FullTaskProgressResponse>('get_task_full_progress', { taskId })
    }
    return request<FullTaskProgressResponse>(`/processing/full-progress/${taskId}`)
  },

  // 重置任务（可选删除已导入记录）
  reset: async (taskId: string, deleteRecords: boolean): Promise<ProcessingTask> => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<ProcessingTask>('reset_processing_task', { taskId, deleteRecords })
    }
    return request<ProcessingTask>(`/processing/reset/${taskId}`, {
      method: 'POST',
      body: JSON.stringify({ delete_records: deleteRecords }),
    })
  },

  // 监听进度事件（Tauri 模式）
  onProgress: (callback: (progress: ProcessingProgress) => void): Promise<UnlistenFn> => {
    return listen<ProcessingProgress>('processing-progress', (event) => {
      callback(event.payload)
    })
  },
}

// ============ 批次 API ============

export const batchesApi = {
  list: async (projectId: number) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<Batch[]>('get_batches', { projectId })
    }
    return request<Batch[]>(`/batches/${projectId}`)
  },

  create: async (projectId: number, batchNumber: string, fileCount: number) => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<Batch>('create_batch', { projectId, batchNumber, fileCount })
    }
    return request<Batch>('/batches/', {
      method: 'POST',
      body: JSON.stringify({ project_id: projectId, batch_number: batchNumber, file_count: fileCount }),
    })
  },

  // 获取项目所有批次（带详情统计）
  listWithStats: async (projectId: number): Promise<BatchDetailResponse[]> => {
    return await invoke<BatchDetailResponse[]>('get_project_batches_with_stats', { projectId })
  },

  // 获取批次详情
  getDetails: async (projectId: number, batchNumber: string): Promise<BatchDetailResponse> => {
    return await invoke<BatchDetailResponse>('get_batch_details', { projectId, batchNumber })
  },

  // 撤回整个批次
  rollback: async (projectId: number, batchNumber: string): Promise<RollbackResult> => {
    return await invoke<RollbackResult>('rollback_batch', { projectId, batchNumber })
  },

  // 撤回单个文件
  rollbackFile: async (projectId: number, batchNumber: string, fileName: string): Promise<RollbackResult> => {
    return await invoke<RollbackResult>('rollback_file', { projectId, batchNumber, fileName })
  },

  // 撤回单个 Sheet
  rollbackSheet: async (projectId: number, batchNumber: string, fileName: string, sheetName: string): Promise<RollbackResult> => {
    return await invoke<RollbackResult>('rollback_sheet', { projectId, batchNumber, fileName, sheetName })
  },
}

// ============ 结果 API ============

export const resultsApi = {
  query: async (
    projectId: number,
    params: {
      page?: number
      page_size?: number
      batch_number?: string
      status?: string
      search?: string
    } = {}
  ) => {
    // 使用 Tauri Commands 查询记录
    const response = await invoke<QueryRecordsResponse>('query_records', {
      projectId,
      page: params.page,
      pageSize: params.page_size,
      batchNumber: params.batch_number,
      status: params.status,
      search: params.search,
    })

    // 转换为 QueryResult 格式（保持向后兼容）
    return {
      records: response.records.map(r => ({
        id: r.id,
        ...r.data,
        raw_data: r.raw_data ?? null,  // 索引格式字符串：1:列1内容;2:列2内容;...
        source_file: r.source_file,
        source_sheet: r.source_sheet,
        batch_number: r.batch_number,
        status: r.status,
      })),
      total: response.total,
      page: response.page,
      page_size: response.page_size,
    } as QueryResult
  },

  update: async (projectId: number, recordId: number, data: Record<string, any>) => {
    return await invoke<ProjectRecord>('update_record', { id: recordId, data })
  },

  delete: async (projectId: number, recordId: number) => {
    await invoke<void>('delete_record', { id: recordId })
  },

  export: async (
    projectId: number,
    format: 'xlsx' | 'csv' = 'xlsx',
    batchNumber?: string
  ): Promise<Blob> => {
    // TODO: 实现导出功能
    throw new Error('导出功能尚未实现')
  },
}

// ============ 记录 API ============

export const recordsApi = {
  insert: async (
    projectId: number,
    data: Record<string, any>,
    options?: {
      sourceFile?: string
      sourceSheet?: string
      rowNumber?: number
      batchNumber?: string
      status?: string
      errorMessage?: string
    }
  ) => {
    return await invoke<ProjectRecord>('insert_record', {
      projectId,
      data,
      sourceFile: options?.sourceFile,
      sourceSheet: options?.sourceSheet,
      rowNumber: options?.rowNumber,
      batchNumber: options?.batchNumber,
      status: options?.status,
      errorMessage: options?.errorMessage,
    })
  },

  insertBatch: async (
    projectId: number,
    records: Record<string, any>[],
    options?: {
      sourceFile?: string
      sourceSheet?: string
      batchNumber?: string
    }
  ) => {
    return await invoke<number>('insert_records_batch', {
      projectId,
      records,
      sourceFile: options?.sourceFile,
      sourceSheet: options?.sourceSheet,
      batchNumber: options?.batchNumber,
    })
  },

  query: async (
    projectId: number,
    params?: {
      page?: number
      pageSize?: number
      batchNumber?: string
      status?: string
      filters?: Record<string, string>
    }
  ) => {
    return await invoke<QueryRecordsResponse>('query_records', {
      projectId,
      page: params?.page,
      pageSize: params?.pageSize,
      batchNumber: params?.batchNumber,
      status: params?.status,
      filters: params?.filters,
    })
  },

  get: async (id: number) => {
    return await invoke<ProjectRecord>('get_record', { id })
  },

  update: async (id: number, data: Record<string, any>) => {
    return await invoke<ProjectRecord>('update_record', { id, data })
  },

  delete: async (id: number) => {
    await invoke<void>('delete_record', { id })
  },

  deleteAll: async (projectId: number) => {
    return await invoke<number>('delete_project_records', { projectId })
  },

  count: async (projectId: number, status?: string) => {
    return await invoke<number>('get_record_count', { projectId, status })
  },

  checkDuplicate: async (projectId: number, dedupValues: Record<string, string>) => {
    return await invoke<number | null>('check_duplicate', { projectId, dedupValues })
  },
}

export const aiServiceApi = {
  // 分析列映射
  analyzeColumnMapping: async (
    aiConfigId: number,
    sheetHeaders: string[],
    fieldDefinitions: FieldDefinition[],
    sampleRows?: string[][]
  ): Promise<ColumnMappingResponse> => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<ColumnMappingResponse>('analyze_column_mapping', {
        aiConfigId,
        sheetHeaders,
        fieldDefinitions,
        sampleRows,
      })
    }
    // HTTP API 回退
    return request<ColumnMappingResponse>('/ai-service/analyze-mapping', {
      method: 'POST',
      body: JSON.stringify({
        ai_config_id: aiConfigId,
        sheet_headers: sheetHeaders,
        field_definitions: fieldDefinitions,
        sample_rows: sampleRows,
      }),
    })
  },

  // AI 辅助生成字段元数据
  generateFieldMetadataWithAI: async (
    aiConfigId: number,
    fieldLabel: string,
    fieldType: string,
    additionalRequirement?: string
  ): Promise<FieldMetadata> => {
    if (USE_TAURI_COMMANDS) {
      return await invoke<FieldMetadata>('ai_generate_field_metadata', {
        aiConfigId,
        fieldLabel,
        fieldType,
        additionalRequirement,
      })
    }
    // HTTP API 回退
    return request<FieldMetadata>('/ai-service/generate-metadata', {
      method: 'POST',
      body: JSON.stringify({
        ai_config_id: aiConfigId,
        field_label: fieldLabel,
        field_type: fieldType,
        additional_requirement: additionalRequirement,
      }),
    })
  },
}

// ============ 统计 API ============

export interface ProjectStatistics {
  total_records: number
  today_records: number
  week_records: number
  month_records: number
  total_tasks: number
  success_tasks: number
  success_rate: number
  last_processed_at: string | null
}

export const statisticsApi = {
  get: async (projectId: number): Promise<ProjectStatistics> => {
    return await invoke<ProjectStatistics>('get_project_statistics', { projectId })
  },
}
