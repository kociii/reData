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
} from '~/types'

// 使用环境变量或默认值来选择后端
// RUST_BACKEND=true 使用 Rust 后端（端口 8001）
// 否则使用 Python 后端（端口 8000）
const USE_RUST_BACKEND = process.env.RUST_BACKEND === 'true' || true // 临时默认使用 Rust 后端
const API_BASE = USE_RUST_BACKEND
  ? 'http://127.0.0.1:8001/api'
  : 'http://127.0.0.1:8000/api'

console.log(`Using ${USE_RUST_BACKEND ? 'Rust' : 'Python'} backend: ${API_BASE}`)

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
  list: () => request<Project[]>('/projects'),

  get: (id: number) => request<Project>(`/projects/${id}`),

  create: (data: CreateProjectRequest) =>
    request<Project>('/projects', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: number, data: UpdateProjectRequest) =>
    request<Project>(`/projects/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: number) =>
    request<void>(`/projects/${id}`, {
      method: 'DELETE',
    }),
}

// ============ 字段 API ============

export const fieldsApi = {
  list: (projectId: number) =>
    request<ProjectField[]>(`/fields/project/${projectId}`),

  create: (data: CreateFieldRequest) =>
    request<ProjectField>('/fields/', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: number, data: Partial<CreateFieldRequest>) =>
    request<ProjectField>(`/fields/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: number) =>
    request<void>(`/fields/${id}`, {
      method: 'DELETE',
    }),

  generateMetadata: (data: GenerateFieldMetadataRequest) =>
    request<FieldMetadata>('/fields/generate-metadata', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
}

// ============ AI 配置 API ============

export const aiConfigsApi = {
  list: () => request<AiConfig[]>('/ai-configs/'),

  get: (id: number) => request<AiConfig>(`/ai-configs/${id}`),

  getDefault: () => request<AiConfig>('/ai-configs/default'),

  create: (data: CreateAiConfigRequest) =>
    request<AiConfig>('/ai-configs/', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: number, data: Partial<CreateAiConfigRequest>) =>
    request<AiConfig>(`/ai-configs/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: number) =>
    request<void>(`/ai-configs/${id}`, {
      method: 'DELETE',
    }),

  testConnection: (id: number) =>
    request<{ success: boolean; message: string; response: string }>(`/ai-configs/test-connection`, {
      method: 'POST',
      body: JSON.stringify({ config_id: id }),
    }),

  setDefault: (id: number) =>
    request<AiConfig>(`/ai-configs/${id}/set-default`, {
      method: 'POST',
    }),
}

// ============ 文件 API ============

export const filesApi = {
  preview: async (file: File): Promise<ExcelPreview> => {
    const formData = new FormData()
    formData.append('file', file)

    const response = await fetch(`${API_BASE}/files/preview`, {
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
  start: (data: StartProcessingRequest) =>
    request<ProcessingTask>('/processing/start', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  pause: (taskId: string) =>
    request<ProcessingTask>(`/processing/pause/${taskId}`, {
      method: 'POST',
    }),

  resume: (taskId: string) =>
    request<ProcessingTask>(`/processing/resume/${taskId}`, {
      method: 'POST',
    }),

  cancel: (taskId: string) =>
    request<ProcessingTask>(`/processing/cancel/${taskId}`, {
      method: 'POST',
    }),

  status: (taskId: string) =>
    request<ProcessingTask>(`/processing/status/${taskId}`),

  list: (projectId: number, status?: string) => {
    const params = new URLSearchParams()
    if (status) params.append('status', status)
    return request<{ tasks: ProcessingTask[]; total: number }>(
      `/processing/list/${projectId}?${params.toString()}`
    )
  },

  // WebSocket 连接
  connectProgress: (taskId: string): WebSocket => {
    return new WebSocket(`ws://127.0.0.1:8000/api/processing/ws/progress/${taskId}`)
  },
}

// ============ 结果 API ============

export const resultsApi = {
  query: (
    projectId: number,
    params: {
      page?: number
      page_size?: number
      batch_number?: string
      status?: string
      search?: string
    } = {}
  ) => {
    const searchParams = new URLSearchParams()
    if (params.page) searchParams.append('page', params.page.toString())
    if (params.page_size) searchParams.append('page_size', params.page_size.toString())
    if (params.batch_number) searchParams.append('batch_number', params.batch_number)
    if (params.status) searchParams.append('status', params.status)
    if (params.search) searchParams.append('search', params.search)

    return request<QueryResult>(
      `/results/${projectId}?${searchParams.toString()}`
    )
  },

  update: (projectId: number, recordId: number, data: Record<string, any>) =>
    request<void>(`/results/${projectId}/${recordId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (projectId: number, recordId: number) =>
    request<void>(`/results/${projectId}/${recordId}`, {
      method: 'DELETE',
    }),

  export: async (
    projectId: number,
    format: 'xlsx' | 'csv' = 'xlsx',
    batchNumber?: string
  ): Promise<Blob> => {
    const params = new URLSearchParams()
    params.append('format', format)
    if (batchNumber) params.append('batch_number', batchNumber)

    const response = await fetch(
      `${API_BASE}/results/export/${projectId}?${params.toString()}`
    )

    if (!response.ok) {
      throw new Error('Failed to export results')
    }

    return response.blob()
  },
}
