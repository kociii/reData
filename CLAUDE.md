# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在此代码库中工作时提供指导。

## 项目概述

**reData** 是一个基于 Tauri 构建的智能数据处理平台桌面应用。它是一个多项目管理系统，允许用户创建不同的项目，每个项目可以自定义需要提取的字段。系统使用 AI 模型自动识别表头，并从数百万个非标准化的 Excel 文件中提取结构化数据。

**核心能力**：
- **多项目管理**：用户可以创建多个独立项目，每个项目有独立的字段定义和数据存储
- **灵活的字段定义**：使用类 Excel 的表格编辑器，轻松定义需要提取的字段
- **AI 列映射分析**：每 Sheet 仅 1 次 AI 调用，分析表头位置和列映射关系
- **本地验证导入**：根据映射结果直接读取数据，使用格式规则验证（节省 99.9% AI 调用）
- **可配置去重**：每个项目可以设置是否去重，以及按哪些字段去重
- **多文件并行处理**：实时进度跟踪
- **本地 SQLite 存储**：每个项目独立存储，完整数据可追溯
- **AI 辅助字段定义**：自动生成英文字段名和提取提示

## 技术栈

**前端**: Nuxt 4.x + TypeScript + Nuxt UI 4.x + Pinia
**桌面框架**: Tauri 2.x
**后端**: Rust + Tauri Commands（零网络开销）🚀
**数据库**: SQLite 3.40+
**AI 集成**: OpenAI SDK（支持 GPT-4、Claude、通过 Ollama 的本地模型）

## 架构

### 通信模式

**当前架构：Tauri Commands 模式（零网络开销）** 🚀

- **前端 → 后端**: 通过 Tauri `invoke()` 直接调用 Rust Commands
  - 零网络开销：直接函数调用，无 HTTP 请求
  - 更快的响应速度：无序列化/反序列化开销
  - 类型安全：Rust 类型系统保证数据一致性

- **后端 → 前端**: 通过 Tauri 事件系统进行实时进度更新
  - `app.emit("processing-progress", &event)` 发送进度事件
  - 前端通过 `listen('processing-progress', callback)` 接收
  - 零延迟、原生桌面性能

### Tauri Commands 实现进度

**已实现**：
- ✅ 项目管理 Commands（`commands/projects.rs`）
  - get_projects, get_project, create_project, update_project, delete_project
- ✅ 字段管理 Commands（`commands/fields.rs`）
  - get_fields, get_all_fields, create_field, update_field, delete_field, restore_field, generate_field_metadata
- ✅ AI 配置 Commands（`commands/ai_configs.rs`）
  - get_ai_configs, get_ai_config, get_default_ai_config, create_ai_config, update_ai_config, delete_ai_config, set_default_ai_config, test_ai_connection
- ✅ AI 服务 Commands（`commands/ai_service.rs`）
  - analyze_column_mapping, ai_generate_field_metadata
- ✅ 记录管理 Commands（`commands/records.rs`）
  - insert_record, insert_records_batch, query_records, get_record, update_record, delete_record, delete_project_records, get_record_count, check_duplicate
- ✅ AI 工具函数（`commands/ai_utils.rs`）
  - call_ai, extract_json（共享 AI 调用工具）
- ✅ Excel 解析 Commands（`commands/excel.rs`）
  - get_excel_sheets, preview_excel
- ✅ 任务管理 Commands（`commands/tasks.rs`）
  - create_processing_task, get_processing_task, list_processing_tasks, update_task_status, create_batch, get_batches
- ✅ 数据处理 Commands（`commands/processing.rs`）
  - start_processing, pause_processing_task, resume_processing_task, cancel_processing_task
  - 实现两阶段处理流程（AI 列映射 + 本地验证导入）
  - 使用 Tauri 事件系统推送进度

**总计**：36 个 Tauri Commands 已实现 🚀

### Tauri 参数命名约定 ⚠️

**重要**：Tauri 2.x 的 `#[tauri::command]` 宏会将 Rust 的蛇形命名参数（snake_case）自动转换为驼峰命名（camelCase）。

**规则**：
- Rust 端定义：`api_url: String`
- 前端调用时：`invoke('create_ai_config', { apiUrl: '...' })`

**常见转换**：
| Rust 参数名 | 前端调用键名 |
|------------|-------------|
| `api_url` | `apiUrl` |
| `model_name` | `modelName` |
| `api_key` | `apiKey` |
| `is_default` | `isDefault` |
| `project_id` | `projectId` |
| `field_name` | `fieldName` |
| `field_type` | `fieldType` |
| `is_required` | `isRequired` |
| `is_dedup_key` | `isDedupKey` |
| `additional_requirement` | `additionalRequirement` |
| `validation_rule` | `validationRule` |
| `extraction_hint` | `extractionHint` |
| `ai_config_id` | `aiConfigId` |
| `sheet_headers` | `sheetHeaders` |
| `field_definitions` | `fieldDefinitions` |
| `sample_rows` | `sampleRows` |
| `source_file` | `sourceFile` |
| `source_sheet` | `sourceSheet` |
| `row_number` | `rowNumber` |
| `batch_number` | `batchNumber` |
| `error_message` | `errorMessage` |
| `page_size` | `pageSize` |
| `dedup_values` | `dedupValues` |

**注意**：前端 TypeScript 类型定义仍使用蛇形命名（与数据库字段一致），只在 `invoke()` 调用时转换为驼峰命名。

### 两阶段数据处理方案

**核心思想**：每 Sheet 仅 1 次 AI 调用，节省 99.9% Token

**阶段一：AI 列映射分析**
1. 读取前 10 行样本数据
2. AI 识别表头位置（第 1-10 行，或无表头）
3. AI 分析每一列与项目字段的匹配关系
4. 返回列映射和置信度

**阶段二：本地验证导入**
1. 根据列映射直接读取对应列
2. 使用格式验证规则检查数据（正则表达式）
3. 逐行导入到数据库

## 数据库架构

**核心表**：
1. **projects** - 项目表（名称、描述、去重配置）
2. **project_fields** - 字段定义表（字段名、类型、验证规则、AI 提示）
3. **project_records** - 统一记录表（JSON `data` 列，以 field_id 为 key）
4. **processing_tasks** - 任务跟踪（UUID、状态枚举）
5. **ai_configs** - AI 配置（加密的 API 密钥）
6. **batches** - 批次统计

**关键特性**：
- 使用 JSON 统一存储方案：`data` 列以 `field_id` 为 key（如 `{"3": "张三", "5": "13800138000"}`）
- 字段改名、调序零成本（只改 `project_fields` 表，记录不动）
- 支持 `json_extract()` 进行字段级查询和去重检查
- 根据去重配置动态构建 `json_extract` 查询

## 开发命令

### 快速开始

```bash
# 安装依赖
cd redata-app
npm install

# 启动 Tauri 开发模式（推荐）🚀
npm run tauri:dev

# 生产构建
npm run tauri:build
```

### 数据库

- 数据库文件：`redata-app/src-tauri/data/app.db`
- 首次运行时自动创建
- 重置数据库：删除 `data/app.db` 并重启应用

## 重要约定

### 批次处理
- 处理前文件被复制到 `history/batch_XXX/`（批次号自动递增）
- 原始文件保持不变，实现可追溯性

### 错误处理
- 失败的行会被记录但不会停止处理
- 错误消息存储在 `error_message` 字段
- AI API 失败会触发自动重试（最多 3 次）

### 空行检测
- 连续 10 个空行后跳到下一个 sheet
- 遇到非空行时计数器重置

### 多 Sheet 处理
- 每个 sheet 独立进行表头识别
- **有表头**：从表头行 + 1 开始处理
- **无表头**：从第 1 行开始处理
- Sheet 名称记录在 `source_sheet` 字段

## 文件组织

### 前端（`redata-app/app/`）
- `pages/` - 页面组件（Nuxt 自动路由）
  - `index.vue` - 项目列表页
  - `project/[id]/fields.vue` - 字段定义页
  - `project/[id]/processing.vue` - 数据处理页
  - `project/[id]/results.vue` - 结果展示页
  - `project/[id]/settings.vue` - 项目设置页
  - `settings.vue` - AI 配置管理页
- `stores/` - Pinia 状态管理（projectStore, fieldStore, processingStore, resultStore, configStore, tabStore）
- `utils/api.ts` - API 客户端（使用 Tauri invoke）

### 后端（`redata-app/src-tauri/src/`）
- `commands/` - Tauri Commands（前端调用入口）
  - `projects.rs` - 项目管理 Commands ✅
  - `fields.rs` - 字段管理 Commands ✅
  - `ai_configs.rs` - AI 配置 Commands ✅
  - `ai_service.rs` - AI 服务 Commands ✅
  - `ai_utils.rs` - AI 工具函数（共享 call_ai, extract_json）✅
  - `records.rs` - 记录管理 Commands ✅
  - `excel.rs` - Excel 解析 Commands ✅
  - `tasks.rs` - 任务管理 Commands ✅
  - `processing.rs` - 数据处理 Commands（两阶段处理 + 事件系统）✅
- `backend/` - 核心业务逻辑（DDD 架构）
  - `domain/` - 领域层（实体、值对象、仓储接口）
  - `application/` - 应用层（用例、DTO）
  - `infrastructure/` - 基础设施层（数据库、加密、日志）
  - `presentation/` - 表现层（HTTP API，已弃用）

### Python 后端（已弃用）
- `redata-app/backend/` - Python FastAPI 后端（保留用于参考）
  - `src/redata/services/` - 业务逻辑（ai_client, validator, excel_parser, extractor, storage）

## Rust AI 集成

项目使用 `async-openai` 库（v0.24）进行 AI 调用，完全兼容 OpenAI API 规范。

### 支持的平台

- OpenAI (GPT-4, GPT-4o, etc.)
- Anthropic Claude（通过兼容层）
- Ollama 本地模型
- vLLM 自托管
- 其他 OpenAI 兼容 API

### 核心功能

- **自定义 API Base URL**：支持连接 Ollama、vLLM 等自托管服务
- **JSON 结构化输出**：通过 `ResponseFormat` 类型实现
- **流式响应**（可选）：使用 `create_stream()` 方法
- **内置 429 重试**：HTTP 429 自动重试，指数退避
- **可配置超时**：通过自定义 `reqwest::Client`

### async-openai 使用示例

**自定义 Base URL（支持 Ollama）**：
```rust
use async_openai::{Client, config::OpenAIConfig};

let config = OpenAIConfig::new()
    .with_api_base("http://localhost:11434/v1")
    .with_api_key("ollama");
let client = Client::with_config(config);
```

**Chat Completions 调用**：
```rust
use async_openai::types::{CreateChatCompletionRequestArgs, ChatCompletionRequestUserMessageArgs};

let request = CreateChatCompletionRequestArgs::default()
    .model("gpt-4")
    .messages([ChatCompletionRequestUserMessageArgs::default()
        .content("你好")
        .build()?.into()])
    .temperature(0.7)
    .build()?;

let response = client.chat().create(request).await?;
```

**JSON 结构化输出**：
```rust
.request(.response_format(ResponseFormat {
    r#type: ChatCompletionResponseFormatType::JsonObject,
}))
```

### 相关文件

- `src-tauri/src/backend/services/ai_client.rs` - AI 客户端服务（待实现）
- `src-tauri/src/backend/infrastructure/config/crypto.rs` - API 密钥加密
- `src-tauri/src/backend/infrastructure/persistence/models/ai_config.rs` - AI 配置模型

### AI 配置数据结构

```rust
pub struct AiConfig {
    pub id: i32,
    pub name: String,
    pub api_url: String,      // 支持 OpenAI/Ollama/vLLM 等
    pub model_name: String,
    pub api_key: String,      // AES-256-GCM 加密存储
    pub temperature: f32,
    pub max_tokens: i32,
    pub is_default: bool,
}
```

## 安全考虑

- API 密钥使用 AES-256-GCM 加密存储
- 数据库文件保持本地，永不上传到云端
- 使用参数化查询防止 SQL 注入
- 验证文件路径以防止目录遍历攻击

## 已知问题和修复

### 字段操作导致应用重启 (v2.5.0)

**问题**：开发模式下，新建、编辑或删除字段时，应用会"闪退"并自动重启

**根本原因**：
Tauri dev server 的热重载文件监听器监控了 `src-tauri/` 整个目录。每次数据库写操作（INSERT/UPDATE/DELETE）都会修改 `data/app.db` 和 `data/app.db-journal` 文件，Tauri 将其误判为源码变更，触发应用重建重启。

**表现**：
- 终端日志显示 `Info File src-tauri/data/app.db-journal changed. Rebuilding application...`
- Rust 端命令实际执行成功（数据库操作已完成），但随后应用被重启

**修复**：
1. 创建 `src-tauri/.taurignore` 文件，排除数据库文件的监听：
   ```
   data/*.db
   data/*.db-journal
   data/*.db-wal
   data/*.db-shm
   ```
2. Rust 端 `create_field` 和 `update_field` 函数添加空字符串处理，将空字符串自动转为 `None`
3. 前端 API 调用时使用驼峰命名（如 `fieldName` 而非 `field_name`）

**相关文件**：
- `src-tauri/.taurignore` - Tauri 文件监听排除规则（关键修复）
- `src-tauri/src/commands/fields.rs` - 添加空值处理逻辑
- `app/utils/api.ts` - 修正 Tauri invoke 参数命名

**代码示例**（Rust 端空值处理）：
```rust
// 处理可选字段：空字符串转为 None
let additional_requirement = additional_requirement
    .and_then(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) });
let validation_rule = validation_rule
    .and_then(|s| if s.trim().is_empty() { None } else { Some(s) });
let extraction_hint = extraction_hint
    .and_then(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) });
```

### WebSocket 连接问题 (v2.4.1) - 已弃用

> **注意**：WebSocket 已在 v2.6.0 被 Tauri 事件系统替代。以下内容仅作历史记录。

**问题**：文件导入时 WebSocket 连接错误，task_id 为空

**原因**：后端在后台异步任务中才生成 task_id，导致 API 返回空字符串

**修复**：在 `start_processing` 函数中提前生成 task_id 和 batch_number

### Tauri 事件系统 (v2.6.0)

**架构变化**：WebSocket → Tauri Events

**优势**：
- 零延迟：原生桌面 IPC 通信
- 更简单：无需管理连接/重连
- 更可靠：Tauri 框架原生支持

**使用方式**：

**后端发送事件**（`processing.rs`）：
```rust
app.emit("processing-progress", &ProcessingEvent {
    event: "row_processed".to_string(),
    task_id: Some(task_id.clone()),
    processed_rows: Some(processed),
    total_rows: Some(total),
    ..Default::default()
})?;
```

**前端监听事件**（`processing.ts`）：
```typescript
import { listen } from '@tauri-apps/api/event'

unlistenProgress = await processingApi.onProgress((data) => {
  handleProgressEvent(data)
})
```

**事件类型**：
- `file_start` / `file_complete` - 文件开始/完成
- `sheet_start` / `sheet_complete` - Sheet 开始/完成
- `ai_analyzing` - AI 分析中
- `column_mapping` - 列映射完成
- `row_processed` - 行处理进度（每 10 行节流）
- `completed` / `error` / `warning` - 任务状态

## 开发进度

**v2.6.0（当前版本）**：
- ✅ 文件处理 Commands（Excel 解析：get_excel_sheets, preview_excel）
- ✅ 任务管理 Commands（6 个命令）
- ✅ 数据处理 Commands（两阶段处理：start_processing, pause/resume/cancel）
- ✅ Tauri 事件系统（替代 WebSocket，实时进度推送）
- ✅ 前端 Store 重构（processing.ts 使用 Tauri events）
- ✅ 前端页面适配（processing.vue）
- ✅ 总计 36 个 Tauri Commands 已实现

**v2.5.0**：
- ✅ 实现 Tauri Commands 模式（项目管理）
- ✅ 零网络开销的前后端通信
- ✅ 字段管理 Commands（7 个命令）
- ✅ AI 配置 Commands（8 个命令）
- ✅ AI 服务 Commands（2 个命令）
- ✅ 记录管理 Commands（9 个命令，JSON 统一存储）
- ✅ 前端 API 迁移到 Tauri invoke

**v2.4.0**：
- ✅ 完成所有 10 个开发阶段（Python 后端）
- ✅ 全局标签页功能
- ✅ AI 辅助字段定义
- ✅ UI 优化（卡片布局、固定表头）

**v2.3.0**：
- ✅ 两阶段处理方案（节省 99.9% Token）
- ✅ 本地数据验证器

## 文档

- [README.md](README.md) - 项目说明
- [DDD_ARCHITECTURE.md](redata-app/backend/DDD_ARCHITECTURE.md) - DDD 架构设计
- [RUST_MIGRATION_PLAN.md](redata-app/backend/RUST_MIGRATION_PLAN.md) - Rust 迁移计划
- `prd/` 目录 - 完整的产品需求和设计文档
