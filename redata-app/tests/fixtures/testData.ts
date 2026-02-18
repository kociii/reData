import type { Project, ProjectField, AiConfig, ProcessingTask, QueryResult } from '~/types'

// 测试项目数据
export const mockProjects: Project[] = [
  {
    id: 1,
    name: '测试项目1',
    description: '这是测试项目1的描述',
    dedup_enabled: true,
    dedup_fields: ['phone'],
    dedup_strategy: 'skip',
    created_at: '2024-01-01T00:00:00',
    updated_at: null
  },
  {
    id: 2,
    name: '测试项目2',
    description: '这是测试项目2的描述',
    dedup_enabled: false,
    dedup_fields: null,
    dedup_strategy: 'skip',
    created_at: '2024-01-02T00:00:00',
    updated_at: null
  }
]

// 测试字段数据
export const mockFields: ProjectField[] = [
  {
    id: 1,
    project_id: 1,
    field_name: 'name',
    field_label: '姓名',
    field_type: 'text',
    is_required: true,
    is_dedup_key: false,
    additional_requirement: null,
    validation_rule: null,
    extraction_hint: '提取用户姓名',
    display_order: 0,
    created_at: '2024-01-01T00:00:00'
  },
  {
    id: 2,
    project_id: 1,
    field_name: 'phone',
    field_label: '手机号',
    field_type: 'phone',
    is_required: true,
    is_dedup_key: true,
    additional_requirement: null,
    validation_rule: '^1[3-9]\\d{9}$',
    extraction_hint: '提取11位手机号',
    display_order: 1,
    created_at: '2024-01-01T00:00:00'
  },
  {
    id: 3,
    project_id: 1,
    field_name: 'email',
    field_label: '邮箱',
    field_type: 'email',
    is_required: false,
    is_dedup_key: false,
    additional_requirement: null,
    validation_rule: '^[\\w\\.-]+@[\\w\\.-]+\\.\\w+$',
    extraction_hint: '提取邮箱地址',
    display_order: 2,
    created_at: '2024-01-01T00:00:00'
  }
]

// 测试 AI 配置数据
export const mockAiConfigs: AiConfig[] = [
  {
    id: 1,
    name: 'GPT-4',
    api_url: 'https://api.openai.com/v1',
    model_name: 'gpt-4',
    api_key: 'sk-test-key',
    temperature: 0.7,
    max_tokens: 2000,
    is_default: true,
    created_at: '2024-01-01T00:00:00',
    updated_at: null
  },
  {
    id: 2,
    name: 'Claude',
    api_url: 'https://api.anthropic.com/v1',
    model_name: 'claude-3-opus',
    api_key: 'sk-ant-test-key',
    temperature: 0.5,
    max_tokens: 4000,
    is_default: false,
    created_at: '2024-01-02T00:00:00',
    updated_at: null
  }
]

// 测试处理任务数据
export const mockProcessingTask: ProcessingTask = {
  id: 'task-001',
  task_id: 'task-001',
  project_id: 1,
  status: 'processing',
  total_files: 1,
  processed_files: 0,
  total_rows: 100,
  processed_rows: 50,
  success_count: 48,
  error_count: 2,
  batch_number: 'batch_001',
  message: '处理中...',
  file_name: 'test.xlsx',
  sheet_name: 'Sheet1',
  started_at: '2024-01-01T00:00:00',
  duration: 30,
  progress: 50
}

// 测试查询结果
export const mockQueryResult: QueryResult = {
  records: [
    {
      id: 1,
      name: '张三',
      phone: '13800138001',
      email: 'zhangsan@example.com',
      source_file: 'test.xlsx',
      source_sheet: 'Sheet1',
      row_number: 2,
      batch_number: 'batch_001',
      status: 'success'
    },
    {
      id: 2,
      name: '李四',
      phone: '13800138002',
      email: 'lisi@example.com',
      source_file: 'test.xlsx',
      source_sheet: 'Sheet1',
      row_number: 3,
      batch_number: 'batch_001',
      status: 'success'
    }
  ],
  total: 2,
  page: 1,
  page_size: 50
}

// 创建项目请求
export const createProjectRequest = {
  name: '新项目',
  description: '新项目描述',
  dedup_enabled: true,
  dedup_fields: ['phone'],
  dedup_strategy: 'skip'
}

// 创建字段请求
export const createFieldRequest = {
  project_id: 1,
  field_name: 'address',
  field_label: '地址',
  field_type: 'text',
  is_required: false,
  is_dedup_key: false
}

// 创建 AI 配置请求
export const createAiConfigRequest = {
  name: '新配置',
  api_url: 'https://api.example.com/v1',
  model_name: 'model-1',
  api_key: 'test-key',
  temperature: 0.5,
  max_tokens: 2000,
  is_default: false
}
