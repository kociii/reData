# 智能数据处理平台 - 实施计划

## 开发进度 Todolist

| 阶段 | 任务 | 状态 | 负责人 | 预计工期 | 实际工期 | 备注 |
|------|------|------|--------|----------|----------|------|
| Phase 1 | 项目初始化 | ✅ 已完成 | - | 1天 | 1天 | Nuxt 4 + Tauri 2 + Nuxt UI 4 |
| Phase 2 | 数据库和基础服务 | ✅ 已完成 | - | 2-3天 | 2天 | Python FastAPI + SQLAlchemy |
| Phase 3 | AI 集成和 Excel 解析 | ✅ 已完成 | - | 2天 | 1天 | 两阶段处理方案 |
| Phase 4 | 前端基础架构 | ✅ 已完成 | - | 1天 | 0.5天 | 布局、API 客户端、状态管理 |
| Phase 5 | 前端 - 项目管理 | ✅ 已完成 | - | 2天 | 0.5天 | 项目列表、创建、切换 |
| Phase 6 | 前端 - 字段定义 | ✅ 已完成 | - | 2天 | 1天 | AI 辅助字段生成工作流 |
| Phase 7 | 前端 - 处理界面 | ✅ 已完成 | - | 3天 | 0.5天 | 文件处理、进度显示 |
| Phase 8 | 前端 - 结果页面 | ✅ 已完成 | - | 2天 | 0.5天 | 数据展示、编辑、导出 |
| Phase 9 | UI 优化 | ✅ 已完成 | - | 1天 | 1天 | 全局标签页、卡片布局、固定表头 |
| Phase 10 | 后端 API 集成 | ✅ 已完成 | - | 1天 | 0.5天 | AI 字段生成接口 |

**状态说明**：
- ⬜ 未开始
- 🔄 进行中
- ✅ 已完成
- ⚠️ 有问题
- 🔴 已阻塞

**总进度**：10/10 (100%)

**架构说明**：
- 后端: Python FastAPI (http://127.0.0.1:8000)
- 前端通过 HTTP API 与后端通信
- Tauri 主要用于桌面壳和文件系统访问

## 已完成工作详情

### Phase 1: 项目初始化 ✅

**完成时间**: 2026-02-17

**已完成任务**:
- [x] 创建 Nuxt 4 项目
- [x] 安装 Tauri 2 CLI 并初始化
- [x] 安装 Nuxt UI 4.x
- [x] 配置项目结构 (app/pages 目录结构)
- [x] 配置 Pinia 状态管理
- [x] 修复 Node.js 23 兼容性问题

**技术栈确认**:
- 前端: Nuxt 4.3.1 + Vue 3.5.28 + Nuxt UI 4.4.0
- 桌面框架: Tauri 2.10.0
- 状态管理: Pinia 3.0.4

### Phase 2: 数据库和基础服务 ✅

**完成时间**: 2026-02-17

**架构变更**: 从 Rust 后端迁移到 Python FastAPI

**已完成任务**:
- [x] 创建 Python 后端项目结构
- [x] 实现数据库模型 (SQLAlchemy)
  - `Project` - 项目模型
  - `ProjectField` - 字段定义模型
  - `ProcessingTask` - 任务模型
  - `AiConfig` - AI 配置模型
  - `Batch` - 批次模型
- [x] 实现 Pydantic Schemas
- [x] 实现 API 路由
  - `/api/projects` - 项目 CRUD
  - `/api/fields` - 字段 CRUD
  - `/api/ai-configs` - AI 配置 CRUD
- [x] 数据库自动初始化

**技术栈**:
- 后端: FastAPI + SQLAlchemy + uvicorn
- 数据库: SQLite (data/app.db)
- 包管理: uv

### Phase 3: AI 集成和 Excel 解析 ✅

**完成时间**: 2026-02-17

**重要变更**：采用"AI 列映射分析 + 本地验证导入"的两阶段处理，节省 99.9% 的 AI 调用。

**已完成任务**:
- [x] 实现 AI 客户端服务 (`services/ai_client.py`)
  - OpenAI SDK 集成
  - 字段元数据生成 Prompt
  - **AI 列映射分析 Prompt**（每 Sheet 仅 1 次 AI 调用）
  - 错误重试机制 (最多 3 次)
  - 超时控制 (30 秒)
  - 测试连接功能

- [x] 实现 Excel 解析服务 (`services/excel_parser.py`)
  - 使用 openpyxl 读取 Excel
  - 遍历 sheets 和行
  - 空行检测 (连续 10 行跳过)
  - Sheet 预览功能
  - **按列索引读取数据**

- [x] 实现数据存储服务 (`services/storage.py`)
  - 动态表创建/管理
  - 记录 CRUD 操作
  - 去重处理（skip/update/merge）
  - 分页查询
  - 数据导出（xlsx/csv）

- [x] **实现数据验证器 (`services/validator.py`)** - 新增
  - 必填字段验证
  - 类型验证（phone, email, url, date）
  - 自定义正则验证
  - 数据标准化（手机号去空格、邮箱转小写等）

- [x] 实现数据提取协调器 (`services/extractor.py`)
  - **两阶段处理流程**：
    - 阶段一：AI 列映射分析（每 Sheet 1 次）
    - 阶段二：本地验证导入（无 AI 调用）
  - 批次管理
  - 进度跟踪
  - 暂停/恢复/取消控制
  - 错误记录保存

- [x] 扩展 API 路由
  - `/api/files` - 文件上传、预览、批次管理
  - `/api/processing` - 启动/暂停/恢复/取消处理、WebSocket 进度
  - `/api/results` - 结果查询、更新、删除、导出

- [x] 增强 AI 配置 API
  - `/api/ai-configs/test-connection` - 测试连接
  - `/api/ai-configs/{id}/set-default` - 设置默认配置
  - `/api/ai-configs/default` - 获取默认配置

**新增文件**:
- `backend/src/redata/services/ai_client.py`
- `backend/src/redata/services/excel_parser.py`
- `backend/src/redata/services/storage.py`
- `backend/src/redata/services/validator.py` **(新增)**
- `backend/src/redata/services/extractor.py`
- `backend/src/redata/api/files.py`
- `backend/src/redata/api/processing.py`
- `backend/src/redata/api/results.py`

**Token 节省对比**:
- 旧方案：1 个 Sheet 有 1000 行 = 1000 次 AI 调用
- 新方案：1 个 Sheet = 1 次 AI 调用
- **节省 99.9% 的 AI 调用**

## 下一步开发计划

### Phase 10: 后端 API 集成 (进行中)

**预计工期**: 1天

**任务清单**:
- [ ] 实现 AI 字段生成接口
  - 接收：field_label, field_type, additional_requirement
  - 返回：field_name (英文), validation_rule, extraction_hint
- [ ] 更新字段定义 API
  - 支持新的字段结构
  - 集成 AI 生成逻辑
- [ ] 前端集成 AI 字段生成
  - 在 `fields.vue` 中调用 AI 接口
  - 显示生成进度
  - 处理生成错误

### Phase 11: 测试和打包

**预计工期**: 2天

**任务清单**:
- [ ] 功能测试（各种表格格式、并行处理、暂停/恢复、去重）
- [ ] 性能优化（AI 调用频率、数据库批量插入、前端渲染）
- [ ] 错误处理完善
- [ ] 配置打包选项
- [ ] 打包应用
- [ ] 测试安装包

## Phase 4-8: 前端开发完成详情

### Phase 4: 前端基础架构 ✅

**完成时间**: 2026-02-17

**已完成任务**:
- [x] 创建 TypeScript 类型定义 (`app/types/index.ts`)
- [x] 创建 API 客户端 (`app/utils/api.ts`)
- [x] 创建 Pinia Stores:
  - `project.ts` - 项目状态管理
  - `field.ts` - 字段状态管理
  - `config.ts` - AI 配置状态管理
  - `processing.ts` - 处理任务状态管理
  - `result.ts` - 结果数据状态管理
- [x] 创建应用布局 (`app/layouts/default.vue`)

### Phase 5: 前端 - 项目管理 ✅

**完成时间**: 2026-02-17

**已完成任务**:
- [x] 实现项目列表页 (`app/pages/index.vue`)
  - 项目卡片展示
  - 创建项目对话框
  - 删除确认对话框
  - 加载和空状态
- [x] 实现项目详情页 (`app/pages/project/[id].vue`)
  - 项目信息展示
  - 统计卡片
  - 功能标签页
  - 文件上传对话框

### Phase 6: 前端 - 字段定义 ✅

**完成时间**: 2026-02-17

**已完成任务**:
- [x] 实现字段定义页 (`app/pages/project/[id]/fields.vue`)
  - 模态框编辑模式（替代内联编辑）
  - 用户输入：字段名称、字段类型、补充提取要求
  - AI 生成：字段名(英文)、验证规则、提取提示
  - 字段添加/编辑/删除
  - 字段类型选择
  - 必填字段配置

### Phase 7: 前端 - 处理界面 ✅

**完成时间**: 2026-02-17

**已完成任务**:
- [x] 实现处理页面 (`app/pages/project/[id]/processing.vue`)
  - 任务列表展示
  - 进度条显示
  - 暂停/恢复/取消功能
  - 实时日志显示
  - 状态图标和颜色

### Phase 8: 前端 - 结果页面 ✅

**完成时间**: 2026-02-17

**已完成任务**:
- [x] 实现结果页面 (`app/pages/project/[id]/results.vue`)
  - 动态列数据表格
  - 分页功能
  - 搜索功能
  - 行内编辑功能
  - 导出对话框
- [x] 实现设置页面 (`app/pages/settings.vue`)
  - AI 配置管理
  - 测试连接
  - 设为默认
  - 应用设置

**新增前端文件**:
- `app/types/index.ts` - TypeScript 类型定义
- `app/utils/api.ts` - API 客户端
- `app/stores/project.ts` - 项目状态管理
- `app/stores/field.ts` - 字段状态管理
- `app/stores/config.ts` - 配置状态管理
- `app/stores/processing.ts` - 处理状态管理
- `app/stores/result.ts` - 结果状态管理
- `app/stores/tab.ts` - 标签页状态管理
- `app/layouts/default.vue` - 应用布局
- `app/pages/index.vue` - 项目列表页
- `app/pages/project/[id].vue` - 项目详情页
- `app/pages/project/[id]/fields.vue` - 字段定义页
- `app/pages/project/[id]/processing.vue` - 处理页面
- `app/pages/project/[id]/results.vue` - 结果页面
- `app/pages/settings.vue` - 设置页面
- `app/components/ProgressBar.vue` - 进度条组件

### Phase 9: UI 优化 ✅

**完成时间**: 2026-02-17

**已完成任务**:
- [x] 实现全局标签页功能
  - 创建 `app/stores/tab.ts` 标签页状态管理
  - 修改 `app/layouts/default.vue` 添加全局标签栏
  - "项目列表"固定为第一个不可关闭标签
  - 项目标签可以打开/关闭/切换
  - 设置页面作为独立标签打开
- [x] 优化项目卡片布局
  - 移除卡片信息字段显示
  - 按钮收起为下拉菜单
  - 卡片可点击直接打开项目
  - 弹性网格布局 `grid-cols-[repeat(auto-fill,minmax(280px,1fr))]`
  - 简化"新建项目"卡片为横向布局
- [x] 优化结果页面
  - 固定表头 `sticky top-0`
  - 默认每页 50 条数据
  - 无数据时也显示表头
  - 修复分页组件使用
- [x] 优化设置页面
  - AI 配置字段改为普通输入框
  - 改善表单间距 `space-y-5`
  - 设置作为独立标签打开

### Phase 10: 后端 API 集成 ✅

**完成时间**: 2026-02-17

**已完成任务**:
- [x] 添加 `additional_requirement` 字段到数据库模型
  - 更新 `ProjectField` 模型添加 `additional_requirement` 字段
  - 执行数据库迁移脚本
- [x] 更新 Pydantic schemas
  - 更新 `ProjectFieldBase` 添加 `additional_requirement` 字段
  - 新增 `GenerateFieldMetadataRequest` 和 `GenerateFieldMetadataResponse` schemas
- [x] 更新 AI 客户端服务
  - 更新 `FieldMetadata` 数据类添加 `validation_rule` 字段
  - 修改 `generate_field_metadata` 方法接收 `additional_requirement` 参数
  - 优化 AI prompt 生成验证规则
- [x] 添加字段元数据生成 API 端点
  - 在 `fields.py` 中添加 `POST /api/fields/generate-metadata` 端点
  - 集成 AI 客户端生成字段元数据
- [x] 更新前端类型定义
  - 更新 `GenerateFieldMetadataRequest` 添加 `additional_requirement`
  - 更新 `FieldMetadata` 添加 `validation_rule`
- [x] 前端集成 AI 字段生成功能
  - 在 `fields.vue` 的 `saveField` 方法中调用 AI 生成接口
  - 使用 AI 生成的 `field_name`, `validation_rule`, `extraction_hint`

**新增/修改文件**:
- `backend/src/redata/models/project.py` - 添加 `additional_requirement` 字段
- `backend/src/redata/models/schemas.py` - 添加新的 schemas
- `backend/src/redata/services/ai_client.py` - 更新 AI 生成逻辑
- `backend/src/redata/api/fields.py` - 添加生成接口
- `backend/migrate_add_additional_requirement.py` - 数据库迁移脚本
- `app/types/index.ts` - 更新类型定义
- `app/pages/project/[id]/fields.vue` - 集成 AI 生成

## 下一步开发计划

### Phase 11: 测试和打包

**预计工期**: 2天

**任务清单**:
- [ ] 功能测试（各种表格格式、并行处理、暂停/恢复、去重）
- [ ] 性能优化（AI 调用频率、数据库批量插入、前端渲染）
- [ ] 错误处理完善
- [ ] 配置打包选项
- [ ] 打包应用
- [ ] 测试安装包

## Context（项目背景）

本项目旨在构建一个灵活的智能数据处理平台，解决不同业务场景下的表格数据结构化提取问题。用户拥有数百万条存储在 Excel 文件中的非结构化数据，这些数据来源多样、格式不统一（表头位置不固定、有无表头不确定、多 Sheet 等），且不同业务场景需要提取不同的字段。

**核心特点**：
- **多项目管理**：用户可以创建不同的项目，每个项目独立管理数据和配置
- **灵活的字段定义**：使用类 Excel 的表格编辑器，轻松定义需要提取的字段
- **AI 驱动提取**：通过 AI 大模型智能识别表头并提取自定义字段
- **可配置去重**：每个项目可以设置是否去重，以及按哪些字段去重
- **简单易用**：面向普通人设计，无需编程或复杂配置

系统将构建为本地桌面应用，使用 Tauri 框架，提供可视化的项目管理、字段定义、处理界面和结果查询界面，支持并行处理多个文件，实时展示处理进度，并将提取结果存储在本地 SQLite 数据库中。

## 核心需求总结

### 1. 系统架构
- **桌面应用框架**：Tauri 2.x
- **前端技术栈**：Nuxt 3.18+ + TypeScript
- **UI 组件库**：Nuxt UI 3.x (基于 Reka UI 和 Tailwind CSS)
- **状态管理**：Pinia (Nuxt 内置)
- **数据库**：SQLite 3.40+
- **AI 集成**：OpenAI SDK（支持兼容接口）

### 2. 主要界面

#### 项目管理页面
- 项目列表展示（卡片式）
- 项目创建/编辑/删除
- 项目切换
- 项目模板选择（默认模板：客户信息提取）
- 项目配置导入/导出

#### 字段定义页面
- 类 Excel 表格编辑器
- 字段属性配置（字段名、显示名、类型、必填、验证规则、AI 提示）
- 字段拖拽排序
- 去重配置（启用/禁用、去重字段、去重策略）
- Prompt 预览

#### 处理界面
- **布局**：左右分栏
  - **左侧**：处理中的文件列表
    - 显示文件名、进度、状态
    - 支持选择查看详情
  - **右侧**：选中文件的详细处理过程
    - **左侧区域**：整个 Sheet 的预览（表格形式）
    - **右侧区域**：当前正在处理的行的提取结果（根据项目字段动态显示）
- **功能**：
  - 显示当前项目名称和提取字段
  - 选择文件夹或文件进行处理
  - 支持并行处理多个文件
  - 暂停/恢复处理
  - 取消处理任务
- **进度显示**：
  - 行数进度（当前行/总行数）
  - 百分比进度条
  - 成功/失败统计
  - 处理速度（行/分钟）

#### 结果页面
- 展示当前项目的所有数据（动态列，根据项目字段定义）
- 支持编辑功能（直接修改字段值）
- 支持导出功能（Excel/CSV）
- 支持筛选和搜索（按来源文件、批次、日期等）
- 分页展示

#### 设置页面
- AI 配置管理
- 系统设置

### 3. 核心处理流程

1. **项目选择**：
   - 用户选择或创建项目
   - 系统加载项目的字段定义和去重配置

2. **文件导入**：
   - 用户选择文件夹或文件
   - 系统复制文件到 `history/batch_XXX/` 目录（批次号递增）
   - 原文件保持不变

3. **AI 列映射分析**（每 Sheet 仅 1 次）：
   - 读取每个 Sheet 的前 10 行
   - 提交给 AI 模型分析表头位置和列映射关系
   - 返回：`{header_row, column_mappings, confidence}`

4. **本地验证导入**（无 AI 调用）：
   - 根据列映射读取对应列的数据
   - 使用格式验证规则检查数据有效性
   - 逐行导入到数据库
   - 连续 10 个空行则跳过当前 Sheet

5. **数据存储**：
   - 根据项目去重配置处理重复数据
   - 存储到 SQLite 数据库（独立表存储动态字段）
   - 记录原始内容、来源文件、来源 Sheet、批次号

6. **失败处理**：
   - 记录错误信息
   - 继续处理下一行
   - 最后展示失败统计

### 4. AI 配置管理
- 支持多个 AI 配置预设
- 可配置：API URL、Model 名称、API Key、温度、最大 Token
- 设置默认配置
- 处理时自动使用默认配置

## 关键里程碑

| 里程碑 | 完成时间 | 交付物 |
|--------|----------|--------|
| M1: 基础环境搭建 | 第 1 天 | 可运行的 Tauri + Nuxt 应用 |
| M2: 数据库和 AI 集成 | 第 5 天 | 数据库表结构、AI 客户端、Excel 解析器 |
| M3: 后端 API 完成 | 第 6 天 | 所有 Tauri Commands 实现 |
| M4: 前端界面完成 | 第 15 天 | 所有前端页面和组件实现 |
| M5: 测试和发布 | 第 17 天 | 可发布的安装包 |

## 风险和注意事项

### 1. AI 调用成本
- **风险**: 百万级数据调用成本较高
- **应对**: 提供本地模型选项（Ollama）或批量处理优化

### 2. AI 提取准确率
- **风险**: 复杂格式可能识别不准确
- **应对**: 提供手动校验和修正功能（结果页面编辑功能）

### 3. 性能瓶颈
- **风险**: 大文件处理可能卡顿
- **应对**: 使用流式处理、分批加载、异步处理

### 4. 错误处理
- **风险**: 需要完善的错误日志和重试机制
- **应对**: 记录详细的错误信息，实现自动重试（最多 3 次）

### 5. 数据安全
- **风险**: API Key 需要加密存储
- **应对**: 使用 AES-256-GCM 加密存储 API Key

### 6. 动态表管理
- **风险**: SQLite 不支持 DROP COLUMN，删除字段需要重建表
- **应对**: 实现表重建逻辑，确保数据不丢失

## 总结

本计划详细描述了智能表格数据提取系统的完整实施方案，包括 9 个开发阶段和 5 个关键里程碑。整个项目预计需要 17 天完成。

**核心技术栈**：
- 前端：Nuxt 3.18+ + TypeScript + Nuxt UI 3.x + Pinia
- 桌面框架：Tauri 2.x
- 后端：Rust + SQLite + OpenAI SDK
- 关键库：calamine（Excel）、rusqlite（数据库）、reqwest（HTTP）、tokio（异步）

**关键特性**：
- 多项目管理
- 灵活的字段定义（类 Excel 编辑器）
- AI 辅助字段定义（自动生成字段名和提取提示）
- 动态表结构（每个项目独立表）
- 并行处理多个文件
- 实时进度更新
- 暂停/恢复/取消控制
- 可配置去重（按任意字段组合）
- 可视化处理界面
- 结果编辑和导出

**技术细节参考**: 所有技术实现细节请参考 dev.md 文档
