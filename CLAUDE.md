# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在此代码库中工作时提供指导。

## 项目概述

**reData** 是一个基于 Tauri 构建的智能数据处理平台桌面应用。它是一个多项目管理系统，允许用户创建不同的项目，每个项目可以自定义需要提取的字段。系统使用 AI 模型自动识别表头，并从数百万个非标准化的 Excel 文件中提取结构化数据。

**核心能力**：
- **多项目管理**：用户可以创建多个独立项目，每个项目有独立的字段定义和数据存储
- **灵活的字段定义**：使用类 Excel 的表格编辑器，轻松定义需要提取的字段
- **AI 驱动的表头识别**：自动识别表头位置（处理不同行位置的表头或无表头情况）
- **智能字段提取**：根据项目字段定义，从非结构化 Excel 数据中智能提取任意自定义字段
- **可配置去重**：每个项目可以设置是否去重，以及按哪些字段去重
- **多文件并行处理**：实时进度跟踪
- **本地 SQLite 存储**：每个项目独立存储，完整数据可追溯
- **AI 辅助字段定义**：自动生成英文字段名和提取提示

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
- `project.rs` - 项目的 CRUD 操作
- `field.rs` - 字段定义的 CRUD 操作
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
  2. 根据项目字段定义动态生成 AI Prompt
  3. 处理数据行 → AI 提取项目定义的字段
  4. 连续 10 个空行后跳过 sheet
- `storage.rs` - SQLite 操作，动态表创建和管理，根据项目去重配置处理重复

### 状态管理（Pinia）

**主要 store**：
- `projectStore` - 项目列表、当前项目、项目 CRUD
- `fieldStore` - 字段定义、字段编辑
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

**核心表**：

1. **projects** - 项目表，包含项目名称、描述、去重配置
2. **project_fields** - 项目字段定义表，包含字段名、显示名称、类型、验证规则、AI 提取提示
3. **project_{id}_records** - 动态创建的项目数据表，每个项目一个独立的表，表结构根据项目字段定义动态生成
4. **processing_tasks** - 任务跟踪，UUID 主键，状态枚举（pending/processing/paused/completed/cancelled）
5. **ai_configs** - AI 模型配置，加密的 API 密钥，is_default 标志
6. **batches** - 批次统计（batch_001、batch_002...）

**关键特性**：
- 每个项目创建独立的数据表（`project_{id}_records`）
- 表结构根据项目字段定义动态生成
- 支持动态添加/删除字段（ALTER TABLE 或重建表）
- 根据项目去重配置创建相应的 UNIQUE 索引

**关键索引**：
- 项目数据表根据去重配置动态创建索引
- `idx_task_status` on processing_tasks(status) - 活动任务查询
- `idx_project_id` on project_fields(project_id) - 字段查询

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

根据项目去重配置动态处理：

```rust
// 单字段去重
conn.execute(
    "INSERT OR IGNORE INTO project_1_records (...) VALUES (...)",
    params![...],
)?;

// 多字段组合去重
// 创建 UNIQUE 索引：CREATE UNIQUE INDEX idx_dedup ON project_1_records(phone, email);
```

### AI Prompt 模板

**表头识别**：
- 提交前 5 行给 AI
- AI 返回表头行号（1-5）或 0（如果没有表头）
- 以 JSON 格式返回字段列表（如果没有表头则为空数组）

**数据提取**：
- 系统根据项目字段定义动态生成 AI Prompt
- **有表头**：提交"表头:值"对，AI 根据项目字段定义以 JSON 格式提取字段
- **无表头**：直接提交原始行数据，AI 从非结构化内容中提取项目定义的字段

**字段定义示例**（客户信息提取项目）：
- **姓名**：支持中文（张三）、英文（John）、称呼（李先生、王总）
- **手机号**：仅 11 位数字
- **地区**：从地址字段提取，或从公司名称推断（例如："北京XX公司" → "北京市"）
- **邮箱**：标准邮箱格式

**注意**：不同项目可以定义完全不同的字段，AI Prompt 会根据项目字段定义动态生成。

## 文件组织

### 前端结构

- `pages/` - 页面组件（Nuxt 3 自动路由）
  - `index.vue` - 项目列表页（首页）
  - `project/[id].vue` - 项目详情页
  - `project/[id]/fields.vue` - 字段定义页
  - `project/[id]/processing.vue` - 数据处理页
  - `project/[id]/results.vue` - 结果展示页
  - `settings.vue` - 设置页
- `components/` - 可复用组件
  - `ProjectCard.vue` - 项目卡片
  - `FieldEditor.vue` - 字段编辑器（类 Excel 表格）
  - `FileList.vue` - 文件列表
  - `SheetPreview.vue` - Sheet 预览
  - `ExtractionResult.vue` - 提取结果
  - `ProgressBar.vue` - 进度条
- `stores/` - Pinia stores（project、field、processing、result、config）
- `types/` - TypeScript 类型定义

### 后端结构

- `src-tauri/src/commands/` - Tauri 命令处理器（暴露给前端）
  - `project.rs` - 项目管理
  - `field.rs` - 字段定义
  - `file.rs` - 文件操作
  - `processing.rs` - 数据处理
  - `config.rs` - AI 配置
  - `result.rs` - 结果查询
- `src-tauri/src/services/` - 业务逻辑
  - `excel_parser.rs` - Excel 解析
  - `ai_client.rs` - AI 客户端
  - `extractor.rs` - 数据提取
  - `storage.rs` - 数据存储（动态表管理）
- `src-tauri/src/models/` - 数据模型（Project、Field、Task、Config、Record）
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
- 错误消息存储在项目数据表的 `error_message` 字段
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

`prd/` 目录中的完整文档（v2.2.0）：
- `prd.md` - 产品需求和业务逻辑（446 行，专注于业务需求）
- `design.md` - UI/UX 设计及 ASCII 图表（1115 行，详细界面设计）
- `plan.md` - 实施计划（346 行，9 个阶段开发计划）
- `dev.md` - 技术细节和架构（1736 行，完整技术实现）
- `README.md` - 文档索引（305 行，文档导航）

**文档职责**：
- **prd.md**：业务需求（What & Why），不包含技术细节
- **design.md**：界面设计（How - UI/UX），详细的界面布局和交互流程
- **plan.md**：开发计划（How - Process），实施步骤和里程碑
- **dev.md**：技术实现（How - Tech），数据库设计、AI Prompt、代码实现

**重要变更**（v2.2.0）：
- prd.md 从 632 行减少到 446 行（减少约 30%）
- 删除所有技术实现细节（已移至 dev.md）
- 添加交叉引用，形成完整的文档体系
- 文档重复率从 50% 降至 < 10%

参考这些文档了解详细的业务规则、UI 规范和实施指导。

## 安全考虑

- API 密钥在存储到 `ai_configs` 表之前必须加密
- 数据库文件（`data/app.db`）保持本地，永不上传到云端
- 使用参数化查询防止 SQL 注入
- 验证文件路径以防止目录遍历攻击