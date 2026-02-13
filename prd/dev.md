# 智能表格数据提取系统 - 技术文档

## 1. 技术栈

### 1.1 前端技术栈
- **框架**: Vue 3.4+
- **语言**: TypeScript 5.0+
- **构建工具**: Vite 5.0+
- **UI 组件库**: Element Plus 2.5+
- **状态管理**: Pinia 2.1+
- **路由**: Vue Router 4.2+
- **图标**: @element-plus/icons-vue

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

#### 2.2.1 提取的数据记录表 (extracted_records)

```sql
CREATE TABLE extracted_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT,                          -- 姓名（支持中文、英文、带称呼）
    phone TEXT UNIQUE,                  -- 手机号（11位数字，唯一，用于去重）
    company TEXT,                       -- 公司
    region TEXT,                        -- 地区（可从地址或公司名推演）
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

**字段说明**:
- `id`: 主键，自增
- `name`: 姓名，支持中文（张三）、英文（John）、带称呼（李先生、王总）
- `phone`: 11位数字手机号，唯一约束，用于自动去重
- `region`: 地区，优先从地址字段提取，也可从公司名称推演
- `email`: 邮箱地址
- `raw_content`: 保存原始数据，便于后续人工校验
- `batch_number`: 批次号，用于追溯数据来源
- `status`: 记录处理状态，便于筛选失败记录

#### 2.2.2 处理任务表 (processing_tasks)

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

#### 2.2.3 AI 配置表 (ai_configs)

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

#### 2.2.4 批次表 (batches)

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

```sql
-- 手机号索引（用于去重和查询）
CREATE INDEX idx_phone ON extracted_records(phone);

-- 批次号索引（用于按批次查询）
CREATE INDEX idx_batch ON extracted_records(batch_number);

-- 任务状态索引（用于查询进行中的任务）
CREATE INDEX idx_task_status ON processing_tasks(status);

-- 默认配置索引（用于快速获取默认配置）
CREATE INDEX idx_config_default ON ai_configs(is_default);

-- 创建时间索引（用于按时间排序）
CREATE INDEX idx_created_at ON extracted_records(created_at);
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
│   │   ├── processing.ts         # 处理任务状态
│   │   ├── config.ts             # AI 配置状态
│   │   └── result.ts             # 结果数据状态
│   ├── views/                    # 页面组件
│   │   ├── ProcessingView.vue    # 处理界面
│   │   ├── ResultView.vue        # 结果页面
│   │   └── SettingsView.vue      # 设置页面
│   ├── components/               # 通用组件
│   │   ├── FileList.vue          # 文件列表组件
│   │   ├── SheetPreview.vue      # Sheet 预览组件
│   │   ├── ExtractionResult.vue  # 提取结果组件
│   │   ├── ProgressBar.vue       # 进度条组件
│   │   └── ConfigDialog.vue      # 配置对话框
│   ├── types/                    # TypeScript 类型定义
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
    redirect: '/processing'
  },
  {
    path: '/processing',
    name: 'Processing',
    component: () => import('@/views/ProcessingView.vue')
  },
  {
    path: '/result',
    name: 'Result',
    component: () => import('@/views/ResultView.vue')
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('@/views/SettingsView.vue')
  }
]
```

#### 4.1.2 状态管理（Pinia Stores）

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
async fn start_processing(files: Vec<String>, config_id: i32) -> Result<String, String>

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
    filter: ResultFilter,
    page: i32,
    page_size: i32
) -> Result<QueryResult, String>

#[tauri::command]
async fn update_record(id: i32, record: Record) -> Result<(), String>

#[tauri::command]
async fn delete_record(id: i32) -> Result<(), String>

#[tauri::command]
async fn export_results(
    filter: ResultFilter,
    format: String
) -> Result<String, String>
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
- 表头识别
- 逐行提取
- 进度回调（通过事件发送到前端）
- 错误处理

**storage.rs - 数据存储服务**:
- SQLite 操作封装
- 数据插入（带去重）
- 数据查询（支持分页和筛选）
- 数据更新/删除
- 事务管理

## 5. 关键技术实现

### 5.1 并行处理实现

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

### 5.2 手机号去重实现

在数据库层面使用 UNIQUE 约束：

```rust
conn.execute(
    "INSERT OR IGNORE INTO extracted_records
     (name, phone, company, region, raw_content, source_file, source_sheet, row_number, batch_number)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    params![name, phone, company, region, raw_content, source_file, source_sheet, row_number, batch_number],
)?;
```

### 5.3 实时进度更新

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

### 5.4 暂停/恢复实现

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

## 6. AI Prompt 设计

### 6.1 表头识别 Prompt

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

### 6.2 数据提取 Prompt

#### 6.2.1 有表头的表格

```
你是一个数据提取专家。请从以下数据中提取指定字段：

原始数据：
{formatted_row_data}  // 格式：表头1:值1; 表头2:值2; ...

请提取以下字段：
- 姓名（支持中文、英文、带称呼如"李先生"、"王总"）
- 手机号（11位数字）
- 公司
- 地区（优先从地址字段提取，也可从公司名称推演，如"北京XX公司"→"北京市"）
- 邮箱

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

#### 6.2.2 无表头的表格

```
你是一个数据提取专家。请从以下原始数据中提取指定字段：

原始数据：
{raw_row_data}  // 格式：值1 | 值2 | 值3 | ...

请提取以下字段：
- 姓名（支持中文、英文、带称呼如"李先生"、"王总"）
- 手机号（11位数字）
- 公司
- 地区（优先从地址字段提取，也可从公司名称推演，如"北京XX公司"→"北京市"）
- 邮箱

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
```

## 7. 性能优化策略

### 7.1 数据库优化
- 使用索引加速查询
- 批量插入减少 I/O
- 使用事务保证一致性
- 定期 VACUUM 优化数据库文件

### 7.2 前端优化
- 虚拟滚动处理大列表
- 分页加载减少内存占用
- 防抖/节流优化搜索
- 懒加载组件

### 7.3 后端优化
- 异步处理避免阻塞
- 连接池复用数据库连接
- 缓存表头识别结果
- 流式处理大文件

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