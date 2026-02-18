# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在此代码库中工作时提供指导。

## 项目概述

**reData** 是一个基于 Tauri 构建的智能数据处理平台桌面应用（v0.1.0）。它是一个多项目管理系统，允许用户创建不同的项目，每个项目可以自定义需要提取的字段。系统使用 AI 模型自动识别表头，并从非标准化的 Excel 文件中提取结构化数据。

**当前版本**: v0.1.0
**技术架构**: Tauri Commands（Rust 后端）

### 核心能力
- **多项目管理**：用户可以创建多个独立项目，每个项目有独立的字段定义和数据存储
- **灵活的字段定义**：使用类 Excel 的表格编辑器，轻松定义需要提取的字段
- **AI 列映射分析**：每 Sheet 仅 1 次 AI 调用，分析表头位置和列映射关系
- **本地验证导入**：根据映射结果直接读取数据，使用格式规则验证（节省 99.9% AI 调用）
- **智能数据清理**：根据字段类型自动清理换行、空格等非标准格式
- **可配置去重**：每个项目可以设置是否去重，以及按哪些字段去重
- **Tauri 事件系统**：实时进度推送，零延迟通信

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Nuxt 4.x + TypeScript + Nuxt UI 4.x + Pinia |
| 桌面框架 | Tauri 2.x |
| 后端 | Rust + Tauri Commands（36 个命令） |
| 数据库 | SQLite 3.40+ |
| AI 集成 | async-openai 0.24（支持 OpenAI、Ollama 等） |

## 架构

### 通信模式

**当前架构：Tauri Commands 模式（零网络开销）**

- **前端 → 后端**: 通过 Tauri `invoke()` 直接调用 Rust Commands
- **后端 → 前端**: 通过 Tauri 事件系统（`app.emit()` / `listen()`）

### Tauri Commands 实现（36 个）

| 模块 | 文件 | 命令 |
|------|------|------|
| 项目管理 | `commands/projects.rs` | get_projects, get_project, create_project, update_project, delete_project |
| 字段管理 | `commands/fields.rs` | get_fields, get_all_fields, create_field, update_field, delete_field, restore_field, generate_field_metadata |
| AI 配置 | `commands/ai_configs.rs` | get_ai_configs, get_ai_config, get_default_ai_config, create_ai_config, update_ai_config, delete_ai_config, set_default_ai_config, test_ai_connection |
| AI 服务 | `commands/ai_service.rs` | analyze_column_mapping, ai_generate_field_metadata |
| AI 工具 | `commands/ai_utils.rs` | call_ai, extract_json（共享函数） |
| 记录管理 | `commands/records.rs` | insert_record, insert_records_batch, query_records, get_record, update_record, delete_record, delete_project_records, get_record_count, check_duplicate |
| Excel 解析 | `commands/excel.rs` | get_excel_sheets, preview_excel |
| 任务管理 | `commands/tasks.rs` | create_processing_task, get_processing_task, list_processing_tasks, update_task_status, create_batch, get_batches |
| 数据处理 | `commands/processing.rs` | start_processing, pause_processing_task, resume_processing_task, cancel_processing_task |

### Tauri 参数命名约定

**重要**：Tauri 2.x 的 `#[tauri::command]` 宏会将 snake_case 自动转换为 camelCase。

| Rust 参数 | 前端调用 |
|-----------|----------|
| `project_id` | `projectId` |
| `field_name` | `fieldName` |
| `api_key` | `apiKey` |
| `is_default` | `isDefault` |

## 数据库架构

**核心表**：
1. **projects** - 项目表（名称、描述、去重配置）
2. **project_fields** - 字段定义表（字段名、类型、验证规则）
3. **project_records** - 记录表（JSON `data` 列，以 field_id 为 key）
4. **processing_tasks** - 任务跟踪（UUID、状态）
5. **ai_configs** - AI 配置（加密 API 密钥）
6. **batches** - 批次统计

**JSON 统一存储**：
- `data` 列以 `field_id` 为 key：`{"3": "张三", "5": "13800138000"}`
- 支持 `json_extract()` 进行字段级查询

## 开发命令

```bash
# 安装依赖
cd redata-app && npm install

# 开发模式
npm run tauri:dev

# 生产构建
npm run tauri:build
```

**数据库位置**：`redata-app/src-tauri/data/app.db`

## 文件组织

```
redata-app/
├── app/                      # 前端代码
│   ├── pages/                # 页面（Nuxt 路由）
│   │   ├── index.vue         # 项目列表
│   │   └── project/[id]/     # 项目页面
│   │       ├── fields.vue    # 字段定义
│   │       ├── processing.vue# 数据处理
│   │       ├── results.vue   # 结果展示
│   │       └── settings.vue  # 项目设置
│   ├── stores/               # Pinia 状态管理
│   └── utils/api.ts          # API 客户端
├── src-tauri/                # Rust 后端
│   └── src/
│       ├── commands/         # Tauri Commands
│       └── backend/          # DDD 架构业务逻辑
```

## 关键技术实现

### 两阶段数据处理

1. **AI 列映射分析**：读取前 10 行样本，AI 识别表头和列映射
2. **本地验证导入**：根据映射读取数据，正则验证，去重检查

### 数据清理机制

| 字段类型 | 清理规则 |
|---------|---------|
| phone | 仅保留数字和 + 号 |
| email | 去除空格、换行，转小写 |
| number/id_card | 仅保留数字和字母 |
| date | 仅保留数字和日期分隔符 |
| 其他 | 压缩连续空白为单个空格 |

### Tauri 事件系统

```rust
// 后端发送
app.emit("processing-progress", &ProcessingEvent { ... })?;
```

```typescript
// 前端监听
listen('processing-progress', (event) => handleProgressEvent(event.payload))
```

### JSON 数据访问

```typescript
// 数据已展开到根级别
// record = { id, "3": "张三", source_file, ... }
{{ record[field.id] || '-' }}  // ✅ 正确
{{ record.data?.[field.id] }}  // ❌ 错误
```

### Vue 响应式更新（Map 中数组）

```typescript
// 创建新数组触发响应式
const newStages = [...stages]
newStages[index] = { ...newStages[index], status }
taskStages.value.set(taskId, newStages)  // ✅
```

## 已知问题修复

### 字段操作导致应用重启
- **原因**：Tauri 监听数据库文件变化
- **修复**：创建 `.taurignore` 排除 `data/*.db` 文件

### 结果页面数据显示
- **原因**：`resultsApi.query` 展开数据到根级别，模板访问错误
- **修复**：`record[field.id]` 而非 `record.data?.[field.id]`

## 文档

- [README.md](README.md) - 项目说明
- [prd/v0.1.0/RELEASE_NOTES.md](prd/v0.1.0/RELEASE_NOTES.md) - v0.1.0 发布说明
- [prd/v0.1.0/](prd/v0.1.0/) - 完整产品文档

---

**版本**: v0.1.0
**发布日期**: 2026-02-18
