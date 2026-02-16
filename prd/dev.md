# 智能数据处理平台 - 技术文档

## 1. 技术栈

### 1.1 前端技术栈
- **框架**: Nuxt 3.18+ (最新稳定版)
- **语言**: TypeScript 5.0+
- **UI 组件库**: Nuxt UI 3.x (基于 Reka UI 和 Tailwind CSS)
- **状态管理**: Pinia 2.1+ (Nuxt 内置支持)
- **路由**: Nuxt Router (自动路由，基于文件系统)
- **构建工具**: Vite 5.0+ (Nuxt 内置)
- **图标**: Nuxt UI Icons (基于 Iconify)
- **CSS 框架**: Tailwind CSS 3.x (Nuxt UI 内置)

### 1.2 桌面应用框架
- **框架**: Tauri 2.x
- **优势**:
  - 轻量级（相比 Electron，体积小 10 倍以上）
  - 原生性能（使用系统 WebView）
  - 安全性高（Rust 内存安全保证）
  - 跨平台支持（Windows、macOS、Linux）
  - 资源占用低

### 1.3 后端技术栈（Tauri Rust 层）
- **语言**: Rust 1.75+
- **核心依赖**:
  - `tauri` 2.x - Tauri 框架核心
  - `calamine` 0.24+ - Excel 文件解析
  - `rusqlite` 0.31+ - SQLite 数据库操作
  - `reqwest` 0.11+ - HTTP 客户端（调用 AI API）
  - `serde` 1.0+ - 序列化/反序列化
  - `serde_json` 1.0+ - JSON 处理
  - `tokio` 1.35+ - 异步运行时
  - `uuid` 1.6+ - 生成唯一任务 ID
  - `chrono` 0.4+ - 日期时间处理

### 1.4 AI 集成
- **SDK**: OpenAI SDK（支持兼容接口）
- **支持的模型**:
  - OpenAI: GPT-4, GPT-4-Turbo, GPT-3.5-Turbo
  - Anthropic: Claude Opus, Claude Sonnet
  - 国产大模型: 通义千问、文心一言、智谱 GLM 等
  - 本地模型: Ollama (Qwen, Llama, Mistral 等)
- **调用方式**: RESTful API (OpenAI 兼容格式)

## 2. 数据库设计

### 2.1 数据库选型
- **数据库**: SQLite 3.40+
- **选型理由**:
  - 无需独立服务器，嵌入式部署
  - 单文件存储，便于备份和迁移
  - 支持事务，保证数据一致性
  - 性能优秀，适合本地应用
  - 跨平台兼容性好

### 2.2 表结构设计

#### 2.2.1 项目表 (projects)

```sql
CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,          -- 项目名称（唯一）
    description TEXT,                   -- 项目描述
    dedup_enabled INTEGER DEFAULT 1,    -- 是否启用去重（0/1）
    dedup_fields TEXT,                  -- 去重字段（JSON 数组，如 ["phone", "email"]）
    dedup_strategy TEXT DEFAULT 'skip', -- 去重策略：skip（跳过）/update（更新）/merge（合并）
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

**字段说明**:
- `id`: 主键，自增
- `name`: 项目名称，唯一约束
- `dedup_enabled`: 是否启用去重功能
- `dedup_fields`: JSON 数组，存储用于去重的字段名列表
- `dedup_strategy`: 去重策略，支持跳过、更新或合并

#### 2.2.2 项目字段定义表 (project_fields)

```sql
CREATE TABLE project_fields (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,        -- 所属项目 ID
    field_name TEXT NOT NULL,           -- 字段名（英文，如 "phone"）
    field_label TEXT NOT NULL,          -- 字段标签（中文，如 "手机号"）
    field_type TEXT NOT NULL,           -- 字段类型：text/number/email/phone/date
    is_required INTEGER DEFAULT 0,      -- 是否必填（0/1）
    validation_rule TEXT,               -- 验证规则（正则表达式或规则描述）
    extraction_hint TEXT,               -- 提取提示（AI 自动生成或用户自定义）
    display_order INTEGER DEFAULT 0,    -- 显示顺序
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);
```

**字段说明**:
- `project_id`: 关联项目 ID，级联删除
- `field_name`: 字段名（标准英文类名），用于数据存储和 API 交互
- `field_label`: 字段标签（用户输入的中文名称），用于 UI 显示
- `field_type`: 字段类型，用于验证和格式化
- `validation_rule`: 自定义验证规则
- `extraction_hint`: AI 自动生成的提取提示，用于指导数据提取，提高准确率

**AI 辅助字段定义**:
当用户创建项目并定义字段时，系统会调用 AI 自动完成以下任务：
1. **生成标准英文字段名**: 根据用户输入的中文字段标签（如"手机号"），AI 生成标准的英文字段名（如"phone"）
2. **生成提取提示**: AI 根据字段标签和类型，自动生成提取提示（如"11位数字的手机号码"）

#### 2.2.3 项目数据表（动态创建）

**设计理念**: 每个项目创建时，根据字段定义动态创建一个独立的数据表。

**表名规则**: `project_{project_id}_records`

**动态创建示例**（客户信息提取项目）:
```sql
CREATE TABLE project_1_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT,                          -- 姓名（根据字段定义动态生成）
    phone TEXT,                         -- 手机号
    company TEXT,                       -- 公司
    region TEXT,                        -- 地区
    email TEXT,                         -- 邮箱
    raw_content TEXT,                   -- 原始内容（完整的行数据）
    source_file TEXT,                   -- 来源文件路径
    source_sheet TEXT,                  -- 来源 Sheet 名称
    row_number INTEGER,                 -- 行号
    batch_number TEXT,                  -- 批次号（batch_001）
    status TEXT DEFAULT 'success',      -- 状态：success/failed
    error_message TEXT,                 -- 错误信息（如果失败）
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

**字段类型映射**:
- `text` → `TEXT`
- `number` → `REAL`
- `email` → `TEXT`
- `phone` → `TEXT`
- `date` → `TEXT` (ISO 8601 格式)

**创建表的 Rust 实现**:
```rust
fn create_project_table(project_id: i32, fields: &[Field]) -> Result<(), String> {
    let table_name = format!("project_{}_records", project_id);

    // 构建字段定义
    let mut field_defs = vec![];
    for field in fields {
        let sql_type = match field.field_type.as_str() {
            "number" => "REAL",
            _ => "TEXT"
        };
        field_defs.push(format!("{} {}", field.field_name, sql_type));
    }

    // 构建 CREATE TABLE 语句
    let create_sql = format!(
        "CREATE TABLE {} (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            {},
            raw_content TEXT,
            source_file TEXT,
            source_sheet TEXT,
            row_number INTEGER,
            batch_number TEXT,
            status TEXT DEFAULT 'success',
            error_message TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        table_name,
        field_defs.join(",\n            ")
    );

    conn.execute(&create_sql, [])?;
    Ok(())
}
```

**字段说明**:
- 前 N 列: 根据项目字段定义动态生成
- `raw_content`: 保存原始数据，便于后续人工校验
- `batch_number`: 批次号，用于追溯数据来源
- `status`: 记录处理状态，便于筛选失败记录

#### 2.2.4 处理任务表 (processing_tasks)

```sql
CREATE TABLE processing_tasks (
    id TEXT PRIMARY KEY,                -- 任务 ID（UUID）
    batch_number TEXT,                  -- 批次号
    file_path TEXT,                     -- 文件路径
    total_sheets INTEGER,               -- 总 Sheet 数
    processed_sheets INTEGER DEFAULT 0, -- 已处理 Sheet 数
    total_rows INTEGER,                 -- 总行数
    processed_rows INTEGER DEFAULT 0,   -- 已处理行数
    success_count INTEGER DEFAULT 0,    -- 成功数量
    failed_count INTEGER DEFAULT 0,     -- 失败数量
    status TEXT DEFAULT 'pending',      -- 状态：pending/processing/paused/completed/cancelled
    config_id INTEGER,                  -- 使用的 AI 配置 ID
    started_at DATETIME,
    completed_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

**字段说明**:
- `id`: 使用 UUID 作为主键，便于分布式场景
- `status`: 支持多种状态，便于任务管理
- `config_id`: 关联 AI 配置，记录使用的模型

#### 2.2.5 AI 配置表 (ai_configs)

```sql
CREATE TABLE ai_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,                 -- 配置名称
    api_url TEXT NOT NULL,              -- API URL
    model_name TEXT NOT NULL,           -- 模型名称
    api_key TEXT NOT NULL,              -- API Key（加密存储）
    temperature REAL DEFAULT 0.7,       -- 温度（0.0-2.0）
    max_tokens INTEGER DEFAULT 1000,    -- 最大 Token
    is_default INTEGER DEFAULT 0,       -- 是否默认配置（0/1）
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

**字段说明**:
- `api_key`: 需要加密存储，保护用户隐私
- `is_default`: 只能有一个默认配置
- `temperature`: 控制模型输出的随机性
- `max_tokens`: 限制单次调用的 Token 消耗

#### 2.2.6 批次表 (batches)

```sql
CREATE TABLE batches (
    batch_number TEXT PRIMARY KEY,      -- 批次号（batch_001）
    file_count INTEGER DEFAULT 0,       -- 文件数量
    total_records INTEGER DEFAULT 0,    -- 总记录数
    success_count INTEGER DEFAULT 0,    -- 成功数量
    failed_count INTEGER DEFAULT 0,     -- 失败数量
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

**字段说明**:
- `batch_number`: 格式为 batch_001, batch_002...
- 用于统计每个批次的处理情况

### 2.3 索引设计

**项目表索引（动态创建）**:

每个项目表根据去重字段和常用查询字段创建索引：

```sql
-- 示例：为客户信息提取项目创建索引

-- 手机号索引（用于去重和查询）
CREATE INDEX idx_project_1_phone ON project_1_records(phone);

-- 批次号索引（用于按批次查询）
CREATE INDEX idx_project_1_batch ON project_1_records(batch_number);

-- 创建时间索引（用于按时间排序）
CREATE INDEX idx_project_1_created_at ON project_1_records(created_at);

-- 状态索引（用于筛选失败记录）
CREATE INDEX idx_project_1_status ON project_1_records(status);
```

**动态索引创建实现**:
```rust
fn create_project_indexes(project_id: i32, dedup_fields: &[String]) -> Result<(), String> {
    let table_name = format!("project_{}_records", project_id);

    // 为去重字段创建索引
    for field in dedup_fields {
        let index_name = format!("idx_project_{}_{}", project_id, field);
        let create_index_sql = format!(
            "CREATE INDEX {} ON {}({})",
            index_name, table_name, field
        );
        conn.execute(&create_index_sql, [])?;
    }

    // 创建通用索引
    conn.execute(
        &format!("CREATE INDEX idx_{}_batch ON {}(batch_number)", table_name, table_name),
        []
    )?;
    conn.execute(
        &format!("CREATE INDEX idx_{}_created_at ON {}(created_at)", table_name, table_name),
        []
    )?;
    conn.execute(
        &format!("CREATE INDEX idx_{}_status ON {}(status)", table_name, table_name),
        []
    )?;

    Ok(())
}
```

**全局索引**:
```sql
-- 任务状态索引（用于查询进行中的任务）
CREATE INDEX idx_task_status ON processing_tasks(status);

-- 默认配置索引（用于快速获取默认配置）
CREATE INDEX idx_config_default ON ai_configs(is_default);

-- 项目字段索引（用于查询项目的字段定义）
CREATE INDEX idx_project_fields ON project_fields(project_id);
```

## 3. 项目结构

```
reData/
├── src/                          # 前端源码
│   ├── main.ts                   # 入口文件
│   ├── App.vue                   # 根组件
│   ├── router/                   # 路由配置
│   │   └── index.ts
│   ├── stores/                   # Pinia 状态管理
│   │   ├── project.ts            # 项目管理状态
│   │   ├── field.ts              # 字段定义状态
│   │   ├── processing.ts         # 处理任务状态
│   │   ├── config.ts             # AI 配置状态
│   │   └── result.ts             # 结果数据状态
│   ├── views/                    # 页面组件
│   │   ├── ProjectListView.vue   # 项目列表页（首页）
│   │   ├── ProjectDetailView.vue # 项目详情页（包含 3 个 Tab）
│   │   └── SettingsView.vue      # 全局设置页面
│   ├── components/               # 通用组件
│   │   ├── project/              # 项目相关组件
│   │   │   ├── ProjectCard.vue   # 项目卡片
│   │   │   └── ProjectForm.vue   # 项目表单
│   │   ├── field/                # 字段相关组件
│   │   │   ├── FieldEditor.vue   # 字段编辑器（Excel 风格）
│   │   │   └── FieldRow.vue      # 字段行组件
│   │   ├── FileList.vue          # 文件列表组件
│   │   ├── SheetPreview.vue      # Sheet 预览组件
│   │   ├── ExtractionResult.vue  # 提取结果组件
│   │   ├── ProgressBar.vue       # 进度条组件
│   │   └── ConfigDialog.vue      # 配置对话框
│   ├── types/                    # TypeScript 类型定义
│   │   ├── project.ts
│   │   ├── field.ts
│   │   ├── processing.ts
│   │   ├── config.ts
│   │   └── result.ts
│   └── utils/                    # 工具函数
│       └── format.ts
├── src-tauri/                    # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs               # 主入口
│   │   ├── commands/             # Tauri 命令
│   │   │   ├── mod.rs
│   │   │   ├── project.rs        # 项目管理命令
│   │   │   ├── field.rs          # 字段定义命令
│   │   │   ├── file.rs           # 文件操作命令
│   │   │   ├── processing.rs     # 处理命令
│   │   │   ├── config.rs         # 配置命令
│   │   │   └── result.rs         # 结果查询命令
│   │   ├── services/             # 业务逻辑
│   │   │   ├── mod.rs
│   │   │   ├── excel_parser.rs   # Excel 解析服务
│   │   │   ├── ai_client.rs      # AI 客户端服务
│   │   │   ├── extractor.rs      # 数据提取服务
│   │   │   └── storage.rs        # 数据存储服务
│   │   ├── models/               # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── project.rs        # 项目模型
│   │   │   ├── field.rs          # 字段模型
│   │   │   ├── task.rs           # 任务模型
│   │   │   ├── config.rs         # 配置模型
│   │   │   └── record.rs         # 记录模型
│   │   └── db/                   # 数据库
│   │       ├── mod.rs
│   │       └── schema.rs         # 数据库表结构
│   ├── Cargo.toml
│   └── tauri.conf.json
├── history/                      # 历史文件目录
│   ├── batch_001/
│   ├── batch_002/
│   └── ...
├── data/                         # 数据库文件
│   └── app.db
├── prd/                          # 需求文档
│   ├── README.md
│   ├── prd.md
│   ├── design.md
│   ├── plan.md
│   └── dev.md
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## 4. 核心模块设计

### 4.1 前端模块

#### 4.1.1 路由设计

```typescript
// src/router/index.ts
const routes = [
  {
    path: '/',
    name: 'ProjectList',
    component: () => import('@/views/ProjectListView.vue')
  },
  {
    path: '/project/:id',
    name: 'ProjectDetail',
    component: () => import('@/views/ProjectDetailView.vue'),
    children: [
      {
        path: 'result',
        name: 'ProjectResult',
        component: () => import('@/components/ExtractionResult.vue')
      },
      {
        path: 'processing',
        name: 'ProjectProcessing',
        component: () => import('@/components/ProcessingPanel.vue')
      },
      {
        path: 'settings',
        name: 'ProjectSettings',
        component: () => import('@/components/ProjectSettings.vue')
      }
    ]
  },
  {
    path: '/settings',
    name: 'GlobalSettings',
    component: () => import('@/views/SettingsView.vue')
  }
]
```

#### 4.1.2 状态管理（Pinia Stores）

**projectStore (项目管理状态)**:
```typescript
interface ProjectStore {
  projects: Project[]        // 项目列表
  currentProject: Project    // 当前项目

  // Actions
  fetchProjects()            // 获取项目列表
  createProject()            // 创建项目
  updateProject()            // 更新项目
  deleteProject()            // 删除项目
  selectProject()            // 选择项目
}
```

**fieldStore (字段定义状态)**:
```typescript
interface FieldStore {
  fields: Field[]            // 字段列表

  // Actions
  fetchFields()              // 获取字段列表
  addField()                 // 添加字段
  updateField()              // 更新字段
  deleteField()              // 删除字段
  reorderFields()            // 重新排序字段
}
```

**processingStore (处理任务状态)**:
```typescript
interface ProcessingStore {
  tasks: Task[]              // 处理任务列表
  selectedTaskId: string     // 选中的任务 ID
  currentSheet: string       // 当前 Sheet

  // Actions
  startProcessing()          // 开始处理
  pauseProcessing()          // 暂停处理
  resumeProcessing()         // 恢复处理
  cancelProcessing()         // 取消处理
  updateProgress()           // 更新进度
  selectTask()               // 选择任务
}
```

**resultStore (结果状态)**:
```typescript
interface ResultStore {
  records: Record[]          // 记录列表
  total: number              // 总记录数
  page: number               // 当前页码
  pageSize: number           // 每页大小
  filters: FilterOptions     // 筛选条件

  // Actions
  fetchRecords()             // 获取记录
  updateRecord()             // 更新记录
  deleteRecord()             // 删除记录
  exportRecords()            // 导出记录
  setFilters()               // 设置筛选
  changePage()               // 切换页码
}
```

**configStore (配置状态)**:
```typescript
interface ConfigStore {
  configs: AiConfig[]        // 配置列表
  defaultConfigId: number    // 默认配置 ID

  // Actions
  fetchConfigs()             // 获取配置
  saveConfig()               // 保存配置
  deleteConfig()             // 删除配置
  setDefaultConfig()         // 设置默认配置
  testConnection()           // 测试连接
}
```

### 4.2 后端模块（Rust）

#### 4.2.1 Tauri Commands（前端调用的 API）

**project.rs - 项目管理命令**:
```rust
#[tauri::command]
async fn get_projects() -> Result<Vec<Project>, String>

#[tauri::command]
async fn get_project(id: i32) -> Result<Project, String>

#[tauri::command]
async fn create_project(project: Project, fields: Vec<Field>) -> Result<i32, String>
// 创建项目时同时创建数据表和索引

#[tauri::command]
async fn update_project(id: i32, project: Project) -> Result<(), String>

#[tauri::command]
async fn delete_project(id: i32) -> Result<(), String>
// 删除项目时同时删除数据表
```

**field.rs - 字段定义命令**:
```rust
#[tauri::command]
async fn get_project_fields(project_id: i32) -> Result<Vec<Field>, String>

#[tauri::command]
async fn add_field(field: Field) -> Result<i32, String>
// 添加字段时执行 ALTER TABLE ADD COLUMN

#[tauri::command]
async fn generate_field_metadata(field_label: String, field_type: String) -> Result<FieldMetadata, String>
// 调用 AI 生成字段的英文名称和提取提示

#[tauri::command]
async fn update_field(id: i32, field: Field) -> Result<(), String>
// 更新字段定义（不修改表结构，仅更新元数据）

#[tauri::command]
async fn delete_field(id: i32) -> Result<(), String>
// SQLite 不支持 DROP COLUMN，需要重建表

#[tauri::command]
async fn reorder_fields(project_id: i32, field_ids: Vec<i32>) -> Result<(), String>
// 仅更新 display_order，不修改表结构
```

**file.rs - 文件操作命令**:
```rust
#[tauri::command]
async fn select_files() -> Result<Vec<String>, String>

#[tauri::command]
async fn select_folder() -> Result<Vec<String>, String>

#[tauri::command]
async fn copy_to_history(files: Vec<String>) -> Result<String, String>

#[tauri::command]
async fn get_next_batch_number() -> Result<String, String>
```

**processing.rs - 处理命令**:
```rust
#[tauri::command]
async fn start_processing(
    project_id: i32,
    files: Vec<String>,
    config_id: i32
) -> Result<String, String>

#[tauri::command]
async fn pause_processing(task_id: String) -> Result<(), String>

#[tauri::command]
async fn resume_processing(task_id: String) -> Result<(), String>

#[tauri::command]
async fn cancel_processing(task_id: String) -> Result<(), String>

#[tauri::command]
async fn get_processing_status(task_id: String) -> Result<TaskStatus, String>
```

**config.rs - 配置命令**:
```rust
#[tauri::command]
async fn get_configs() -> Result<Vec<AiConfig>, String>

#[tauri::command]
async fn get_default_config() -> Result<AiConfig, String>

#[tauri::command]
async fn save_config(config: AiConfig) -> Result<i32, String>

#[tauri::command]
async fn set_default_config(config_id: i32) -> Result<(), String>

#[tauri::command]
async fn delete_config(config_id: i32) -> Result<(), String>
```

**result.rs - 结果查询命令**:
```rust
#[tauri::command]
async fn query_results(
    project_id: i32,
    filter: ResultFilter,
    page: i32,
    page_size: i32
) -> Result<QueryResult, String>
// 动态查询项目表: SELECT * FROM project_{project_id}_records

#[tauri::command]
async fn update_record(
    project_id: i32,
    record_id: i32,
    field_values: HashMap<String, String>
) -> Result<(), String>
// 动态更新: UPDATE project_{project_id}_records SET field1=?, field2=? WHERE id=?

#[tauri::command]
async fn delete_record(project_id: i32, record_id: i32) -> Result<(), String>
// 动态删除: DELETE FROM project_{project_id}_records WHERE id=?

#[tauri::command]
async fn export_results(
    project_id: i32,
    filter: ResultFilter,
    format: String
) -> Result<String, String>
// 导出项目数据
```

#### 4.2.2 Services（业务逻辑）

**excel_parser.rs - Excel 解析服务**:
- 使用 `calamine` 库解析 Excel 文件
- 支持 .xlsx 和 .xls 格式
- 遍历所有 Sheet
- 读取指定行数据
- 检测空行

**ai_client.rs - AI 客户端服务**:
- 调用 OpenAI 兼容 API
- 实现表头识别 Prompt
- 实现数据提取 Prompt
- 错误重试机制（最多 3 次）
- 超时控制（30 秒）

**extractor.rs - 数据提取服务**:
- 协调整个提取流程
- 根据项目字段定义动态生成 AI Prompt
- 表头识别
- 逐行提取（提取项目定义的字段）
- 进度回调（通过事件发送到前端）
- 错误处理

**storage.rs - 数据存储服务**:
- SQLite 操作封装
- 动态表创建和管理
- 数据插入（动态 SQL，支持任意字段）
- 灵活去重（根据项目配置的去重字段和策略）
- 数据查询（动态查询项目表，支持分页和筛选）
- 数据更新/删除（动态 SQL）
- 事务管理
- ALTER TABLE 操作（添加/删除字段）

## 5. 关键技术实现

### 5.1 动态表创建和管理

**创建项目表**:
```rust
fn create_project_table(project_id: i32, fields: &[Field]) -> Result<(), String> {
    let table_name = format!("project_{}_records", project_id);

    // 构建字段定义
    let mut field_defs = vec![];
    for field in fields {
        let sql_type = match field.field_type.as_str() {
            "number" => "REAL",
            _ => "TEXT"
        };
        let nullable = if field.is_required { "NOT NULL" } else { "" };
        field_defs.push(format!("{} {} {}", field.field_name, sql_type, nullable));
    }

    // 构建 CREATE TABLE 语句
    let create_sql = format!(
        "CREATE TABLE {} (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            {},
            raw_content TEXT,
            source_file TEXT,
            source_sheet TEXT,
            row_number INTEGER,
            batch_number TEXT,
            status TEXT DEFAULT 'success',
            error_message TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        table_name,
        field_defs.join(",\n            ")
    );

    conn.execute(&create_sql, [])?;

    // 创建索引
    create_project_indexes(project_id, &get_dedup_fields(project_id)?)?;

    Ok(())
}
```

**添加字段**:
```rust
fn add_field_to_table(project_id: i32, field: &Field) -> Result<(), String> {
    let table_name = format!("project_{}_records", project_id);
    let sql_type = match field.field_type.as_str() {
        "number" => "REAL",
        _ => "TEXT"
    };

    let alter_sql = format!(
        "ALTER TABLE {} ADD COLUMN {} {}",
        table_name, field.field_name, sql_type
    );

    conn.execute(&alter_sql, [])?;
    Ok(())
}
```

**删除字段**（SQLite 限制，需要重建表）:
```rust
fn remove_field_from_table(project_id: i32, field_name: &str) -> Result<(), String> {
    let table_name = format!("project_{}_records", project_id);
    let temp_table = format!("{}_temp", table_name);

    // 1. 获取当前字段列表（排除要删除的字段）
    let fields = get_project_fields(project_id)?
        .into_iter()
        .filter(|f| f.field_name != field_name)
        .collect::<Vec<_>>();

    // 2. 创建临时表
    create_temp_table(&temp_table, &fields)?;

    // 3. 复制数据
    let field_names = fields.iter()
        .map(|f| f.field_name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    conn.execute(
        &format!("INSERT INTO {} SELECT {} FROM {}", temp_table, field_names, table_name),
        []
    )?;

    // 4. 删除原表
    conn.execute(&format!("DROP TABLE {}", table_name), [])?;

    // 5. 重命名临时表
    conn.execute(&format!("ALTER TABLE {} RENAME TO {}", temp_table, table_name), [])?;

    Ok(())
}
```

**删除项目表**:
```rust
fn drop_project_table(project_id: i32) -> Result<(), String> {
    let table_name = format!("project_{}_records", project_id);
    conn.execute(&format!("DROP TABLE IF EXISTS {}", table_name), [])?;
    Ok(())
}
```

### 5.2 动态数据插入

根据项目字段定义动态构建 INSERT 语句：

```rust
fn insert_record(
    project_id: i32,
    extracted_values: &HashMap<String, String>,
    raw_content: &str,
    source_file: &str,
    batch_number: &str
) -> Result<(), String> {
    let table_name = format!("project_{}_records", project_id);
    let fields = get_project_fields(project_id)?;

    // 构建字段名和占位符
    let mut field_names = vec![];
    let mut placeholders = vec![];
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    for field in &fields {
        field_names.push(field.field_name.clone());
        placeholders.push("?");
        if let Some(value) = extracted_values.get(&field.field_name) {
            values.push(Box::new(value.clone()));
        } else {
            values.push(Box::new(String::new()));
        }
    }

    // 添加固定字段
    field_names.extend(vec!["raw_content", "source_file", "batch_number"]);
    placeholders.extend(vec!["?", "?", "?"]);
    values.push(Box::new(raw_content.to_string()));
    values.push(Box::new(source_file.to_string()));
    values.push(Box::new(batch_number.to_string()));

    // 构建 INSERT 语句
    let insert_sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table_name,
        field_names.join(", "),
        placeholders.join(", ")
    );

    conn.execute(&insert_sql, rusqlite::params_from_iter(values.iter()))?;
    Ok(())
}
```

### 5.3 并行处理实现

使用 Rust 的 `tokio` 异步运行时实现并行处理：

```rust
use tokio::task;

async fn process_files(files: Vec<String>) {
    let mut handles = vec![];

    for file in files {
        let handle = task::spawn(async move {
            process_single_file(file).await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
```

### 5.4 灵活去重实现

根据项目配置的去重字段和策略动态生成去重逻辑：

```rust
// 获取项目的去重配置
let project = get_project(project_id)?;
let dedup_fields: Vec<String> = serde_json::from_str(&project.dedup_fields)?;
let table_name = format!("project_{}_records", project_id);

// 构建去重查询条件
let mut where_clauses = vec![];
let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

params.push(Box::new(project_id));

for field in &dedup_fields {
    where_clauses.push(format!("{} = ?", field));
    if let Some(value) = extracted_values.get(field) {
        params.push(Box::new(value.clone()));
    }
}

let where_clause = where_clauses.join(" AND ");

// 检查是否存在重复记录
let query = format!(
    "SELECT id FROM {} WHERE {}",
    table_name, where_clause
);

let existing_id: Option<i32> = conn.query_row(
    &query,
    rusqlite::params_from_iter(params.iter()),
    |row| row.get(0)
).optional()?;

// 根据去重策略处理
match project.dedup_strategy.as_str() {
    "skip" => {
        // 跳过重复记录
        if existing_id.is_some() {
            return Ok(());
        }
        insert_record(project_id, extracted_values, raw_content, source_file, batch_number)?;
    },
    "update" => {
        // 更新已存在的记录
        if let Some(id) = existing_id {
            update_record(project_id, id, extracted_values)?;
        } else {
            insert_record(project_id, extracted_values, raw_content, source_file, batch_number)?;
        }
    },
    "merge" => {
        // 合并字段（保留非空值）
        if let Some(id) = existing_id {
            merge_record(project_id, id, extracted_values)?;
        } else {
            insert_record(project_id, extracted_values, raw_content, source_file, batch_number)?;
        }
    },
    _ => insert_record(project_id, extracted_values, raw_content, source_file, batch_number)?
}
```

### 5.5 实时进度更新

使用 Tauri 的事件系统：

```rust
// 后端发送事件
app.emit_all("processing-progress", ProgressPayload {
    task_id: task_id.clone(),
    current_row: current_row,
    total_rows: total_rows,
    success_count: success_count,
    failed_count: failed_count,
    current_sheet: sheet_name.clone(),
    processing_speed: speed,
})?;

// 前端监听事件
import { listen } from '@tauri-apps/api/event'

listen('processing-progress', (event) => {
  processingStore.updateProgress(event.payload)
})
```

### 5.6 暂停/恢复实现

使用 `Arc<Mutex<bool>>` 共享暂停状态：

```rust
use std::sync::{Arc, Mutex};

let paused = Arc::new(Mutex::new(false));

// 在处理循环中检查
if *paused.lock().unwrap() {
    tokio::time::sleep(Duration::from_millis(100)).await;
    continue;
}
```

## 6. AI Prompt 动态生成

### 6.1 字段元数据生成 Prompt

当用户创建字段时，调用 AI 自动生成字段的英文名称和提取提示：

```
你是一个数据建模专家。用户正在创建一个数据提取字段，请帮助生成字段的元数据。

字段信息：
- 字段标签（中文）：{field_label}
- 字段类型：{field_type}

请生成以下内容：
1. 标准的英文字段名（遵循 snake_case 命名规范，如 phone_number, company_name）
2. 数据提取提示（简洁描述如何识别和提取这个字段，用于指导 AI 提取数据）

请以 JSON 格式返回：
{
  "field_name": "生成的英文字段名",
  "extraction_hint": "提取提示说明"
}
```

**示例**:
```
输入：
- 字段标签：手机号
- 字段类型：phone

输出：
{
  "field_name": "phone",
  "extraction_hint": "11位数字的中国大陆手机号码"
}

输入：
- 字段标签：公司名称
- 字段类型：text

输出：
{
  "field_name": "company_name",
  "extraction_hint": "公司或组织的完整名称"
}

输入：
- 字段标签：所在地区
- 字段类型：text

输出：
{
  "field_name": "region",
  "extraction_hint": "省市区地址信息，可从地址字段提取，也可从公司名称推断"
}
```

**实现**:
```rust
async fn generate_field_metadata(
    field_label: &str,
    field_type: &str,
    ai_config: &AiConfig
) -> Result<FieldMetadata, String> {
    let prompt = format!(
        "你是一个数据建模专家。用户正在创建一个数据提取字段，请帮助生成字段的元数据。\n\n\
         字段信息：\n\
         - 字段标签（中文）：{}\n\
         - 字段类型：{}\n\n\
         请生成以下内容：\n\
         1. 标准的英文字段名（遵循 snake_case 命名规范，如 phone_number, company_name）\n\
         2. 数据提取提示（简洁描述如何识别和提取这个字段，用于指导 AI 提取数据）\n\n\
         请以 JSON 格式返回：\n\
         {{\n\
           \"field_name\": \"生成的英文字段名\",\n\
           \"extraction_hint\": \"提取提示说明\"\n\
         }}",
        field_label, field_type
    );

    let response = call_ai_api(&prompt, ai_config).await?;
    let metadata: FieldMetadata = serde_json::from_str(&response)?;
    Ok(metadata)
}
```

### 6.2 表头识别 Prompt

表头识别 Prompt 保持通用，不依赖项目字段定义：

```
你是一个表格分析专家。以下是一个 Excel 表格的前 5 行数据：

[第 1 行] {row_1_data}
[第 2 行] {row_2_data}
[第 3 行] {row_3_data}
[第 4 行] {row_4_data}
[第 5 行] {row_5_data}

请分析并判断：
1. 第几行是表头？（返回行号 1-5，如果没有表头则返回 0）
2. 表头包含哪些字段？（返回字段列表，如果没有表头则返回空数组）

请以 JSON 格式返回：
{
  "header_row": 1,  // 1-5 表示表头行号，0 表示无表头
  "headers": ["字段1", "字段2", "字段3", ...]  // 无表头时返回 []
}
```

### 6.3 数据提取 Prompt（动态生成）

根据项目的字段定义动态生成提取 Prompt：

```rust
// 获取项目字段定义
let fields = get_project_fields(project_id)?;

// 构建字段提取说明
let mut field_descriptions = vec![];
for field in &fields {
    let desc = format!(
        "- {}（{}）{}{}",
        field.field_label,
        field.field_type,
        if field.is_required { "【必填】" } else { "" },
        if !field.extraction_hint.is_empty() {
            format!("：{}", field.extraction_hint)
        } else {
            String::new()
        }
    );
    field_descriptions.push(desc);
}

// 构建 JSON 返回格式说明
let mut json_fields = vec![];
for field in &fields {
    json_fields.push(format!("  \"{}\": \"提取的{}\"", field.field_name, field.field_label));
}
```

#### 6.3.1 有表头的表格

```
你是一个数据提取专家。请从以下数据中提取指定字段：

原始数据：
{formatted_row_data}  // 格式：表头1:值1; 表头2:值2; ...

请提取以下字段：
{field_descriptions}  // 动态生成的字段列表

请以 JSON 格式返回：
{
{json_fields}  // 动态生成的 JSON 字段
}

如果某个字段无法提取，请返回空字符串。
```

**示例（客户信息提取项目）**：
```
你是一个数据提取专家。请从以下数据中提取指定字段：

原始数据：
姓名:张三; 联系方式:13800138000; 单位:北京XX科技公司; 邮箱:zhangsan@example.com

请提取以下字段：
- 姓名（text）【必填】：支持中文、英文、带称呼如"李先生"、"王总"
- 手机号（phone）【必填】：11位数字
- 公司（text）
- 地区（text）：优先从地址字段提取，也可从公司名称推演，如"北京XX公司"→"北京市"
- 邮箱（email）

请以 JSON 格式返回：
{
  "name": "提取的姓名",
  "phone": "提取的手机号",
  "company": "提取的公司",
  "region": "提取的地区",
  "email": "提取的邮箱"
}

如果某个字段无法提取，请返回空字符串。
```

#### 6.3.2 无表头的表格

```
你是一个数据提取专家。请从以下原始数据中提取指定字段：

原始数据：
{raw_row_data}  // 格式：值1 | 值2 | 值3 | ...

请提取以下字段：
{field_descriptions}  // 动态生成的字段列表

请以 JSON 格式返回：
{
{json_fields}  // 动态生成的 JSON 字段
}

如果某个字段无法提取，请返回空字符串。
```

### 6.4 Prompt 生成实现

```rust
fn generate_extraction_prompt(
    project_id: i32,
    row_data: &str,
    has_header: bool
) -> Result<String, String> {
    // 获取项目字段定义
    let fields = get_project_fields(project_id)?;

    // 构建字段描述
    let field_descriptions = fields.iter()
        .map(|f| {
            format!(
                "- {}（{}）{}{}",
                f.field_label,
                f.field_type,
                if f.is_required { "【必填】" } else { "" },
                if !f.extraction_hint.is_empty() {
                    format!("：{}", f.extraction_hint)
                } else {
                    String::new()
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    // 构建 JSON 字段
    let json_fields = fields.iter()
        .map(|f| format!("  \"{}\": \"提取的{}\"", f.field_name, f.field_label))
        .collect::<Vec<_>>()
        .join(",\n");

    // 根据是否有表头选择模板
    let template = if has_header {
        format!(
            "你是一个数据提取专家。请从以下数据中提取指定字段：\n\n\
             原始数据：\n{}\n\n\
             请提取以下字段：\n{}\n\n\
             请以 JSON 格式返回：\n{{\n{}\n}}\n\n\
             如果某个字段无法提取，请返回空字符串。",
            row_data, field_descriptions, json_fields
        )
    } else {
        format!(
            "你是一个数据提取专家。请从以下原始数据中提取指定字段：\n\n\
             原始数据：\n{}\n\n\
             请提取以下字段：\n{}\n\n\
             请以 JSON 格式返回：\n{{\n{}\n}}\n\n\
             如果某个字段无法提取，请返回空字符串。",
            row_data, field_descriptions, json_fields
        )
    };

    Ok(template)
}
```

## 7. 性能优化策略

### 7.1 数据库优化
- 使用索引加速查询（为去重字段和常用查询字段创建索引）
- 批量插入减少 I/O
- 使用事务保证一致性
- 定期 VACUUM 优化数据库文件
- 合理设计字段类型（number 使用 REAL，其他使用 TEXT）

### 7.2 前端优化
- 虚拟滚动处理大列表
- 分页加载减少内存占用
- 防抖/节流优化搜索
- 懒加载组件

### 7.3 后端优化
- 异步处理避免阻塞
- 连接池复用数据库连接
- 缓存表头识别结果
- 缓存项目字段定义（减少数据库查询）
- 流式处理大文件
- 动态 Prompt 模板缓存

## 8. 安全性设计

### 8.1 API Key 加密存储
使用 Tauri 的安全存储 API 或自定义加密方案：

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

fn encrypt_api_key(api_key: &str) -> String {
    // 使用 AES-256-GCM 加密
    // ...
}

fn decrypt_api_key(encrypted: &str) -> String {
    // 解密
    // ...
}
```

### 8.2 数据隔离
- 数据库文件本地存储
- 不上传云端
- 支持数据导出和备份

### 8.3 输入验证
- 验证文件格式
- 验证 API 配置
- 防止 SQL 注入（使用参数化查询）

## 9. 错误处理策略

### 9.1 错误分类
- **网络错误**: API 调用失败、超时
- **文件错误**: 文件不存在、格式错误、权限不足
- **数据库错误**: 连接失败、查询错误
- **业务错误**: 数据格式不符、提取失败

### 9.2 错误处理机制
- 记录详细错误日志
- 友好的错误提示
- 自动重试（网络错误）
- 跳过错误继续处理（数据错误）

## 10. 测试策略

### 10.1 单元测试
- Rust 后端逻辑测试
- Vue 组件测试
- 工具函数测试

### 10.2 集成测试
- API 调用测试
- 数据库操作测试
- 文件处理测试

### 10.3 端到端测试
- 完整流程测试
- 性能测试
- 压力测试