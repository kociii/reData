# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在此代码库中工作时提供指导。

## 项目概述

**reData** 是一个基于 Tauri 构建的智能表格数据提取系统桌面应用。它使用 AI 模型自动识别表头，并从数百万个非标准化的 Excel 文件中提取结构化数据（姓名、手机号、公司、地区、邮箱）。

**核心能力**：
- AI 驱动的表头识别（处理不同行位置的表头或无表头情况）
- 从非结构化 Excel 数据中智能提取字段
- 灵活的姓名格式（中文、英文、带称呼如"李先生"、"王总"）
- 11 位手机号提取
- 当地址字段不可用时从公司名称推断地区
- 邮箱提取
- 多文件并行处理，实时进度跟踪
- 按手机号自动去重
- 本地 SQLite 存储，完整数据可追溯

## 技术栈

**前端**: Nuxt 3.18+ + TypeScript + Nuxt UI 3.x
**桌面框架**: Tauri 2.x (Rust 后端)
**数据库**: SQLite 3.40+
**AI 集成**: OpenAI 兼容 API（支持 GPT-4、Claude、通过 Ollama 的本地模型）

**前端特性**：
- Nuxt 3.18+ - 最新稳定版，全栈 Vue 框架
- Nuxt UI 3.x - 基于 Reka UI 和 Tailwind CSS 的直观 UI 库
- 自动路由 - 基于文件系统的路由
- 内置 Pinia - 状态管理
- TypeScript 支持 - 完整的类型安全

**关键 Rust 依赖**：
- `calamine` - Excel 文件解析
- `rusqlite` - SQLite 操作
- `reqwest` - AI API 调用的 HTTP 客户端
- `tokio` - 并行处理的异步运行时
- `uuid` - 任务 ID 生成

## 架构

### Tauri 命令模式

应用使用 Tauri 的命令系统进行前后端通信：

**前端 → 后端**: 通过 `@tauri-apps/api` 调用 Tauri 命令
**后端 → 前端**: 通过 `app.emit_all()` 发送事件进行实时进度更新

**命令模块**（位于 `src-tauri/src/commands/`）：
- `file.rs` - 文件选择，批量复制到 `history/batch_XXX/`
- `processing.rs` - 启动/暂停/恢复/取消处理任务
- `config.rs` - AI 配置的 CRUD 操作
- `result.rs` - 查询/更新/导出提取的记录

### 服务层架构

**服务**（位于 `src-tauri/src/services/`）：
- `excel_parser.rs` - 使用 calamine 读取 Excel 文件，遍历 sheet/行
- `ai_client.rs` - 调用 AI API，带重试逻辑（最多 3 次尝试，30 秒超时）
- `extractor.rs` - 协调提取流程：
  1. 读取前 5 行 → AI 识别表头行
  2. 处理数据行 → AI 提取目标字段
  3. 连续 10 个空行后跳过 sheet
- `storage.rs` - SQLite 操作，使用 `INSERT OR IGNORE` 进行手机号去重

### 状态管理（Pinia）

**三个主要 store**：
- `processingStore` - 活动任务、进度、选中的任务
- `resultStore` - 提取的记录、分页、筛选器
- `configStore` - AI 配置、默认配置

### 实时进度更新

使用 Tauri 的事件系统：

```rust
// 后端发送进度事件
app.emit_all("processing-progress", ProgressPayload { ... })?;
```

```typescript
// 前端监听并更新 UI
listen('processing-progress', (event) => {
  processingStore.updateProgress(event.payload)
})
```

## 数据库架构

**4 个核心表**：

1. **extracted_records** - 提取的数据，字段包括：name（支持中文/英文/称呼）、phone（11 位数字，UNIQUE 用于去重）、company、region（可从公司名称推断）、email、raw_content、来源追踪
2. **processing_tasks** - 任务跟踪，UUID 主键，状态枚举（pending/processing/paused/completed/cancelled）
3. **ai_configs** - AI 模型配置，加密的 API 密钥，is_default 标志
4. **batches** - 批次统计（batch_001、batch_002...）

**关键索引**：
- `idx_phone` on extracted_records(phone) - 快速去重检查
- `idx_batch` on extracted_records(batch_number) - 批次查询
- `idx_task_status` on processing_tasks(status) - 活动任务查询

## 开发命令

### 初始设置

```bash
# 创建 Nuxt 项目
npx nuxi@latest init

# 安装 Tauri CLI
npm install --save-dev @tauri-apps/cli

# 初始化 Tauri
npm run tauri init

# 安装 Nuxt UI
npm install @nuxt/ui

# 安装 Rust 依赖（添加到 src-tauri/Cargo.toml）
# calamine, rusqlite, reqwest, serde, serde_json, tokio, uuid, chrono
```

### 开发

```bash
# 运行开发服务器（热重载）
npm run tauri dev

# 仅前端（用于 UI 开发）
npm run dev

# 生产构建
npm run tauri build
```

### 数据库

数据库文件位置：`data/app.db`
架构初始化：`src-tauri/src/db/schema.rs`

开发期间重置数据库：删除 `data/app.db` 并重启应用。

## 关键实现模式

### 并行文件处理

使用 `tokio::task::spawn` 进行并发文件处理：

```rust
let mut handles = vec![];
for file in files {
    let handle = task::spawn(async move {
        process_single_file(file).await
    });
    handles.push(handle);
}
```

### 暂停/恢复机制

使用 `Arc<Mutex<bool>>` 共享暂停状态：

```rust
let paused = Arc::new(Mutex::new(false));

// 在处理循环中
if *paused.lock().unwrap() {
    tokio::time::sleep(Duration::from_millis(100)).await;
    continue;
}
```

### 手机号去重

在数据库层面使用 UNIQUE 约束处理：

```rust
conn.execute(
    "INSERT OR IGNORE INTO extracted_records (...) VALUES (...)",
    params![...],
)?;
```

### AI Prompt 模板

**表头识别**：
- 提交前 5 行给 AI
- AI 返回表头行号（1-5）或 0（如果没有表头）
- 以 JSON 格式返回字段列表（如果没有表头则为空数组）

**数据提取**：
- **有表头**：提交"表头:值"对，AI 以 JSON 格式提取 name/phone/company/region/email
- **无表头**：直接提交原始行数据，AI 从非结构化内容中提取字段

**字段提取规则**：
- **姓名**：支持中文（张三）、英文（John）、称呼（李先生、王总）
- **手机号**：仅 11 位数字
- **地区**：从地址字段提取，或从公司名称推断（例如："北京XX公司" → "北京市"）
- **邮箱**：标准邮箱格式

## 文件组织

### 前端结构

- `src/views/` - 页面组件（ProcessingView、ResultView、SettingsView）
- `src/components/` - 可复用组件（FileList、SheetPreview、ExtractionResult、ProgressBar）
- `src/stores/` - Pinia stores（processing、result、config）
- `src/types/` - TypeScript 类型定义

### 后端结构

- `src-tauri/src/commands/` - Tauri 命令处理器（暴露给前端）
- `src-tauri/src/services/` - 业务逻辑（Excel 解析、AI 客户端、提取、存储）
- `src-tauri/src/models/` - 数据模型（Task、Config、Record）
- `src-tauri/src/db/` - 数据库架构和连接管理

### 数据目录

- `history/batch_XXX/` - 复制的 Excel 文件（保留原始文件，实现可追溯性）
- `data/app.db` - SQLite 数据库文件

## 重要约定

### 批次处理

- 处理前文件被复制到 `history/batch_XXX/`（批次号自动递增）
- 原始文件保持不变
- 每个批次都有唯一标识符以实现可追溯性

### 错误处理

- 失败的行会被记录但不会停止处理
- 错误消息存储在 `extracted_records.error_message`
- AI API 失败会触发自动重试（最多 3 次）

### 空行检测

- 维护连续空行计数器
- 连续 10 个空行后跳到下一个 sheet
- 遇到非空行时计数器重置

### 多 Sheet 处理

- 每个 sheet 独立进行表头识别
- **如果有表头**：从表头行 + 1 开始处理，使用"表头:值"格式
- **如果无表头**：从第 1 行开始处理，直接提交原始数据给 AI
- Sheet 名称记录在 `source_sheet` 字段
- 文件内的所有 sheet 按顺序处理

## 文档

`prd/` 目录中的完整文档：
- `prd.md` - 产品需求和业务逻辑
- `design.md` - UI/UX 设计及 ASCII 图表
- `plan.md` - 实施计划（9 个阶段，15 天）
- `dev.md` - 技术细节和架构
- `README.md` - 文档索引

参考这些文档了解详细的业务规则、UI 规范和实施指导。

## 安全考虑

- API 密钥在存储到 `ai_configs` 表之前必须加密
- 数据库文件（`data/app.db`）保持本地，永不上传到云端
- 使用参数化查询防止 SQL 注入
- 验证文件路径以防止目录遍历攻击