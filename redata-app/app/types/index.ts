// 全局标签页类型
export interface AppTab {
  id: string           // 'home' 或 'project-{id}'
  label: string        // 显示名称
  path: string         // 当前路由路径（记忆最后访问的子页面）
  closable: boolean    // 是否可关闭
  projectId?: number   // 项目 ID（项目标签页才有）
}

// 项目类型
export interface Project {
  id: number
  name: string
  description: string | null
  created_at: string
  updated_at: string | null
}

export interface CreateProjectRequest {
  name: string
  description?: string
}

export interface UpdateProjectRequest {
  name?: string
  description?: string
}

// 字段类型
export interface ProjectField {
  id: number
  project_id: number
  field_name: string
  field_label: string
  field_type: string
  is_required: boolean
  is_dedup_key: boolean  // 是否参与去重
  is_deleted: boolean    // 软删除标记
  additional_requirement?: string | null
  validation_rule: string | null
  extraction_hint: string | null
  display_order: number
  created_at: string
  deleted_at: string | null  // 删除时间
}

export interface CreateFieldRequest {
  project_id: number
  field_name: string
  field_label: string
  field_type: string
  is_required?: boolean
  is_dedup_key?: boolean
  additional_requirement?: string | null
  validation_rule?: string | null
  extraction_hint?: string | null
}

export interface GenerateFieldMetadataRequest {
  field_label: string
  field_type: string
  additional_requirement?: string | null
}

export interface FieldMetadata {
  field_name: string
  validation_rule: string | null
  extraction_hint: string
}

// AI 列映射分析类型
export interface FieldDefinition {
  field_name: string
  field_label: string
  field_type: string
  additional_requirement?: string | null
}

export interface FieldMapping {
  field_name: string
  column_index: number
  column_header: string
  confidence: number
}

export interface ColumnMappingResponse {
  header_row: number
  mappings: FieldMapping[]
  confidence: number
  unmatched_columns: number[]
}

// AI 配置类型
export interface AiConfig {
  id: number
  name: string
  api_url: string
  model_name: string
  api_key: string
  temperature: number
  max_tokens: number
  is_default: boolean
  created_at: string
  updated_at: string | null
}

export interface CreateAiConfigRequest {
  name: string
  api_url: string
  model_name: string
  api_key: string
  temperature?: number
  max_tokens?: number
  is_default?: boolean
}

// 处理任务类型
export interface ProcessingTask {
  id: string              // 与 task_id 一致，用于 UI 组件 key
  task_id: string
  project_id: number
  status: 'pending' | 'processing' | 'paused' | 'completed' | 'cancelled' | 'error'
  total_files: number
  processed_files: number
  total_rows: number
  processed_rows: number
  success_count: number
  error_count: number
  batch_number: string | null
  source_files?: string[]  // 源文件名列表
  message?: string
  // UI 扩展字段
  file_name?: string
  sheet_name?: string
  started_at?: string
  duration?: number
  progress?: number
  error_message?: string
}

// 待处理文件类型
export interface PendingFile {
  id: string
  path: string
  name: string
  size: number
}

// 日志条目类型
export interface LogEntry {
  time: string
  message: string
  type: 'info' | 'success' | 'warning' | 'error'
  align?: 'left' | 'right'  // 对话式布局：左对齐（系统消息）或右对齐（进度消息）
  category?: 'ai' | 'ai_request' | 'progress' | 'system'  // 日志类别
}

export interface StartProcessingRequest {
  project_id: number
  file_paths: string[]
  ai_config_id?: number
}

// 进度更新类型
export interface ProcessingProgress {
  event: string
  task_id: string
  current_file?: string
  current_sheet?: string
  current_row?: number
  total_rows?: number
  processed_rows?: number
  success_count?: number
  error_count?: number
  speed?: number
  message?: string
  // 列映射事件附加字段
  header_row?: number
  mappings?: Record<string, string>
  confidence?: number
  unmatched_columns?: number[]
}

// 处理阶段类型
export type ProcessingStageStatus = 'pending' | 'active' | 'completed' | 'error'

export interface ProcessingStage {
  key: 'preparing' | 'ai_mapping' | 'importing' | 'done'
  label: string
  status: ProcessingStageStatus
}

// 查询结果类型
export interface QueryResult {
  records: Record<string, any>[]
  total: number
  page: number
  page_size: number
}

// Excel 预览类型
export interface SheetInfo {
  name: string
  row_count: number
  column_count: number
}

export interface ExcelPreview {
  sheets: SheetInfo[]
  rows: any[][]
  sheet_name: string
}

// 项目记录类型（JSON 统一存储，data 以 field_id 为 key）
export interface ProjectRecord {
  id: number
  project_id: number
  data: Record<string, any>  // key 为 field_id 字符串
  raw_data: string | null  // 索引格式字符串：1:列1内容;2:列2内容;...n:列n内容;
  source_file: string | null
  source_sheet: string | null
  row_number: number | null
  batch_number: string | null
  status: string
  error_message: string | null
  created_at: string
  updated_at: string | null
}

export interface QueryRecordsResponse {
  records: ProjectRecord[]
  total: number
  page: number
  page_size: number
}

// 批次类型
export interface Batch {
  id: number
  batch_number: string
  project_id: number
  file_count: number
  record_count: number
  created_at: string
}

// 任务列表响应
export interface ListTasksResponse {
  tasks: ProcessingTask[]
  total: number
}

// ── 数据处理进度类型 v2 ────────────────────────────────────────────────────────

export type SheetPhase = 'waiting' | 'ai_analyzing' | 'importing' | 'done' | 'error'
export type FilePhase = 'waiting' | 'processing' | 'done' | 'error'
export type TaskPhase = 'starting' | 'processing' | 'paused' | 'completed' | 'cancelled' | 'error'

export interface SheetProgress {
  sheetName: string
  phase: SheetPhase
  aiConfidence: number | null    // 0-1，null=尚未完成 AI 分析
  mappingCount: number | null    // 映射成功的字段数，null=尚未完成
  errorMessage: string | null
}

export interface FileProgress {
  fileName: string
  phase: FilePhase
  sheets: SheetProgress[]
  totalRows: number
  successCount: number
  errorCount: number
}

export interface TaskProgress {
  taskId: string
  projectId: number
  batchNumber: string | null
  phase: TaskPhase
  sourceFiles: string[]
  files: FileProgress[]
  totalRows: number
  processedRows: number
  successCount: number
  errorCount: number
  startedAt: string
  completedAt: string | null
}
