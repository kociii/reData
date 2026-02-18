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

- **后端 → 前端**: 通过 WebSocket 进行实时进度更新（待实现）

### Tauri Commands 实现进度

**已实现**：
- ✅ 项目管理 Commands（`commands/projects.rs`）
  - get_projects, get_project, create_project, update_project, delete_project

**待实现**：
- ⏳ 字段管理 Commands
- ⏳ AI 配置 Commands
- ⏳ 文件处理 Commands
- ⏳ 数据处理 Commands
- ⏳ 结果查询 Commands

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
3. **project_{id}_records** - 动态创建的项目数据表（每个项目独立）
4. **processing_tasks** - 任务跟踪（UUID、状态枚举）
5. **ai_configs** - AI 配置（加密的 API 密钥）
6. **batches** - 批次统计

**关键特性**：
- 每个项目创建独立的数据表，表结构根据字段定义动态生成
- 支持动态添加/删除字段（ALTER TABLE 或重建表）
- 根据去重配置创建 UNIQUE 索引

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
- `backend/` - 核心业务逻辑（DDD 架构）
  - `domain/` - 领域层（实体、值对象、仓储接口）
  - `application/` - 应用层（用例、DTO）
  - `infrastructure/` - 基础设施层（数据库、加密、日志）
  - `presentation/` - 表现层（HTTP API，已弃用）

### Python 后端（已弃用）
- `redata-app/backend/` - Python FastAPI 后端（保留用于参考）
  - `src/redata/services/` - 业务逻辑（ai_client, validator, excel_parser, extractor, storage）

## 安全考虑

- API 密钥使用 AES-256-GCM 加密存储
- 数据库文件保持本地，永不上传到云端
- 使用参数化查询防止 SQL 注入
- 验证文件路径以防止目录遍历攻击

## 已知问题和修复

### WebSocket 连接问题 (v2.4.1)

**问题**：文件导入时 WebSocket 连接错误，task_id 为空

**原因**：后端在后台异步任务中才生成 task_id，导致 API 返回空字符串

**修复**：在 `start_processing` 函数中提前生成 task_id 和 batch_number

**相关文件**：
- `backend/src/redata/api/processing.py` - 修复了 task_id 生成逻辑
- `backend/src/redata/services/storage.py` - 添加了智能表结构迁移功能
- `backend/src/redata/models/project.py` - 添加了字段软删除支持

## 开发进度

**v2.5.0（当前版本）**：
- ✅ 实现 Tauri Commands 模式（项目管理）
- ✅ 零网络开销的前后端通信
- ⏳ 其他功能模块迁移到 Tauri Commands

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
