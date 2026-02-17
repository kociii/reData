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

**前端**: Nuxt 4.x + TypeScript + Nuxt UI 4.x
**桌面框架**: Tauri 2.x
**后端**: Python 3.11+ + FastAPI
**数据库**: SQLite 3.40+
**AI 集成**: OpenAI SDK（支持 GPT-4、Claude、通过 Ollama 的本地模型）

**前端特性**：
- Nuxt 4.x - 最新版，全栈 Vue 框架
- Nuxt UI 4.x - 基于 Reka UI 和 Tailwind CSS 的直观 UI 库
- 自动路由 - 基于文件系统的路由
- 内置 Pinia - 状态管理
- TypeScript 支持 - 完整的类型安全

**后端特性**（Python + FastAPI）：
- FastAPI - 现代 Python Web 框架，自动生成 API 文档
- SQLAlchemy - Python ORM，类型安全的数据库操作
- pandas + openpyxl - 强大的 Excel 处理能力
- OpenAI SDK - 官方 AI 集成库
- uvicorn - 高性能 ASGI 服务器

## 架构

### 通信模式

应用使用 HTTP API 进行前后端通信：

**前端 → 后端**: 通过 HTTP API 调用后端服务（http://127.0.0.1:8000）
**后端 → 前端**: 通过 WebSocket 进行实时进度更新

### API 路由（位于 `backend/src/redata/api/`）：

- `projects.py` - 项目的 CRUD 操作 ✅
- `fields.py` - 字段定义的 CRUD 操作 ✅
- `ai_configs.py` - AI 配置的 CRUD 操作 ✅
- `files.py` - 文件上传、预览、批次管理 ✅
- `processing.py` - 启动/暂停/恢复/取消处理任务、WebSocket 进度 ✅
- `results.py` - 查询/更新/导出提取的记录 ✅

### 服务层架构（两阶段处理方案）

**服务**（位于 `backend/src/redata/services/`）：

| 文件 | 功能 | 状态 |
|------|------|------|
| `ai_client.py` | AI 列映射分析、字段元数据生成 | ✅ |
| `validator.py` | 本地格式验证、数据标准化 | ✅ |
| `excel_parser.py` | Excel 读取、按列索引读取 | ✅ |
| `storage.py` | 动态表管理、去重处理 | ✅ |
| `extractor.py` | 两阶段处理协调器 | ✅ |

**两阶段处理流程**：

**阶段一：AI 列映射分析（每 Sheet 仅 1 次 AI 调用）**
1. 读取前 10 行样本数据
2. AI 识别表头位置（第 1-10 行，或无表头）
3. AI 分析每一列与项目字段的匹配关系
4. 返回列映射和置信度

**阶段二：本地验证导入（无 AI 调用）**
1. 根据列映射直接读取对应列
2. 使用格式验证规则检查数据
3. 逐行导入到数据库

**Token 节省对比**：
- 旧方案：1 个 Sheet 有 1000 行 = 1000 次 AI 调用
- 新方案：1 个 Sheet = 1 次 AI 调用
- **节省 99.9% 的 AI 调用**

### 状态管理（Pinia）

**主要 store**：
- `projectStore` - 项目列表、当前项目、项目 CRUD
- `fieldStore` - 字段定义、字段编辑
- `processingStore` - 活动任务、进度、选中的任务
- `resultStore` - 提取的记录、分页、筛选器
- `configStore` - AI 配置、默认配置

### 实时进度更新

使用 WebSocket：

```python
# 后端发送进度事件（FastAPI）
await manager.broadcast(task_id, {
    "event": "row_processed",
    "current_row": 100,
    "total_rows": 500,
    "success_count": 95,
    "error_count": 5
})
```

```typescript
// 前端监听并更新 UI
const ws = new WebSocket('ws://127.0.0.1:8000/api/processing/ws/progress/{task_id}')
ws.onmessage = (event) => {
  const progress = JSON.parse(event.data)
  processingStore.updateProgress(progress)
}
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
# 安装前端依赖
npm install

# 安装后端依赖
cd backend
uv sync
cd ..
```

### 开发

```bash
# 启动后端服务器
cd backend
uv run python run.py

# 启动前端开发服务器（另一个终端）
npm run dev

# 启动 Tauri 开发模式
npm run tauri:dev

# 生产构建
npm run tauri:build
```

### API 文档

后端 API 文档自动生成：
- Swagger UI: http://127.0.0.1:8000/docs
- ReDoc: http://127.0.0.1:8000/redoc

### 数据库

数据库文件位置：`backend/data/app.db`
首次运行时自动创建。

开发期间重置数据库：删除 `backend/data/app.db` 并重启后端服务器。

## 关键实现模式

### 两阶段数据处理（高效模式）

```python
# 阶段一：AI 列映射分析（每 Sheet 仅 1 次）
mapping = await ai_client.analyze_column_mapping(
    sample_rows=sample_rows,  # 前 10 行
    fields=project_fields
)
# 返回: {header_row: 1, column_mappings: {0: "name", 2: "phone"}, confidence: 0.95}

# 阶段二：本地验证导入（无 AI 调用）
for row_num, row_data in parser.iterate_rows(sheet, start_row):
    record = {field_name: row_data[col_idx] for col_idx, field_name in mapping.column_mappings.items()}
    is_valid, errors = validator.validate_record(record, fields)
    if is_valid:
        storage.insert_record(project_id, record, meta)
```

### 本地格式验证

```python
class DataValidator:
    VALIDATORS = {
        "phone": r"^1[3-9]\d{9}$",           # 11位手机号
        "email": r"^[\w\.-]+@[\w\.-]+\.\w+$", # 邮箱
        "url": r"^https?://",                # URL
        "date": r"^\d{4}[-/]\d{1,2}[-/]\d{1,2}$",  # 日期
    }

    def validate(self, value, field):
        # 必填检查
        if field.is_required and not value:
            return False, "必填字段不能为空"
        # 类型验证
        if field.field_type in self.VALIDATORS:
            if not re.match(self.VALIDATORS[field.field_type], str(value)):
                return False, f"格式不正确"
        return True, None
```

### 去重处理

```python
# 根据项目去重配置动态处理
if project.dedup_enabled:
    existing_id = storage.handle_dedup(project, data)
    if existing_id:
        if project.dedup_strategy == "skip":
            return  # 跳过重复
        elif project.dedup_strategy == "update":
            storage.update_record(project_id, existing_id, data)
    else:
        storage.insert_record(project_id, data, meta)
```

### 暂停/恢复机制

```python
class Extractor:
    def __init__(self):
        self.paused = False
        self.cancelled = False

    async def process_sheet(self):
        for row in rows:
            while self.paused:
                await asyncio.sleep(0.1)
            if self.cancelled:
                break
            # 处理行...
```

## 文件组织

### 前端结构

- `app/pages/` - 页面组件（Nuxt 4 自动路由）
  - `index.vue` - 项目列表页（首页）
  - `project/[id].vue` - 项目详情页
  - `project/[id]/fields.vue` - 字段定义页
  - `project/[id]/processing.vue` - 数据处理页
  - `project/[id]/results.vue` - 结果展示页
  - `settings.vue` - 设置页
- `components/` - 可复用组件
- `stores/` - Pinia stores
- `types/` - TypeScript 类型定义

### 后端结构

- `backend/src/redata/api/` - API 路由（FastAPI）✅
  - `projects.py` - 项目管理
  - `fields.py` - 字段定义
  - `files.py` - 文件操作
  - `processing.py` - 数据处理
  - `ai_configs.py` - AI 配置
  - `results.py` - 结果查询
- `backend/src/redata/services/` - 业务逻辑 ✅
  - `ai_client.py` - AI 客户端（列映射分析）
  - `validator.py` - 数据验证器
  - `excel_parser.py` - Excel 解析
  - `extractor.py` - 数据提取协调器
  - `storage.py` - 数据存储（动态表管理）
- `backend/src/redata/models/` - 数据模型
- `backend/src/redata/db/` - 数据库配置

### 数据目录

- `history/batch_XXX/` - 复制的 Excel 文件（保留原始文件，实现可追溯性）
- `backend/data/app.db` - SQLite 数据库文件

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

`prd/` 目录中的完整文档（v2.3.0）：
- `prd.md` - 产品需求和业务逻辑（两阶段处理方案）
- `design.md` - UI/UX 设计及 ASCII 图表
- `plan.md` - 实施计划（9 个阶段，当前进度 3/9）
- `dev.md` - 技术细节和架构（两阶段处理实现）
- `README.md` - 文档索引

**开发进度**：
- ✅ Phase 1: 项目初始化（Nuxt 4 + Tauri 2）
- ✅ Phase 2: 数据库和基础服务（Python FastAPI）
- ✅ Phase 3: AI 集成和 Excel 解析（两阶段处理方案）
- ⬜ Phase 4-9: Tauri Commands、前端界面、测试

**v2.3.0 重要变更**：
- 采用"AI 列映射分析 + 本地验证导入"的两阶段处理
- 每 Sheet 仅 1 次 AI 调用，节省 99.9% 的 Token 消耗
- 新增 `validator.py` 本地数据验证器

## 安全考虑

- API 密钥在存储到 `ai_configs` 表之前必须加密
- 数据库文件（`data/app.db`）保持本地，永不上传到云端
- 使用参数化查询防止 SQL 注入
- 验证文件路径以防止目录遍历攻击