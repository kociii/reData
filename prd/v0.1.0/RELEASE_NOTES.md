# reData v0.1.0 发布说明

**发布日期**: 2026-02-18
**版本**: v0.1.0
**代号**: 首个正式版本

## 版本概述

reData v0.1.0 是项目的首个正式发布版本，实现了完整的智能数据处理功能。本版本采用 **Tauri Commands 架构**，实现了零网络开销的前后端通信，相比传统的 HTTP API 架构性能更优。

## 核心功能

### 1. 多项目管理
- 创建、编辑、删除项目
- 项目级别的数据隔离
- 快速切换项目

### 2. 灵活的字段定义
- 类 Excel 表格编辑器
- 支持多种字段类型：text、phone、email、number、date、id_card
- 字段验证规则配置
- AI 辅助生成字段元数据
- 字段软删除和恢复

### 3. AI 列映射分析
- 每 Sheet 仅 1 次 AI 调用，节省 99.9% Token
- 智能识别表头位置
- 自动分析列与字段的映射关系
- 返回置信度评分

### 4. 本地验证导入
- 基于映射结果直接读取数据
- 格式验证（正则表达式）
- 数据清理（去除换行、空格等）
- 智能去重检查

### 5. 数据搜索
- 全文搜索 JSON 数据
- 300ms 防抖优化

### 6. 实时进度推送
- Tauri 事件系统替代 WebSocket
- 零延迟进度更新
- 任务暂停/恢复/取消

## 技术架构

### 前端
- **Nuxt 4.x** - 全栈 Vue 框架
- **Nuxt UI 4.x** - UI 组件库
- **TypeScript** - 类型安全
- **Pinia** - 状态管理

### 桌面框架
- **Tauri 2.x** - 轻量级桌面应用框架

### 后端（Rust）
- **Tauri Commands** - 零网络开销的前后端通信
- **SeaORM 1.0** - 异步 ORM
- **async-openai 0.24** - OpenAI API 客户端
- **calamine** - Excel 解析

### 数据库
- **SQLite 3.40+** - 本地数据库

## Tauri Commands 实现（36 个）

| 模块 | 命令 | 功能 |
|------|------|------|
| 项目管理 | get_projects, get_project, create_project, update_project, delete_project | 项目 CRUD |
| 字段管理 | get_fields, get_all_fields, create_field, update_field, delete_field, restore_field, generate_field_metadata | 字段管理 + AI 辅助 |
| AI 配置 | get_ai_configs, get_ai_config, get_default_ai_config, create_ai_config, update_ai_config, delete_ai_config, set_default_ai_config, test_ai_connection | AI 配置管理 |
| AI 服务 | analyze_column_mapping, ai_generate_field_metadata | AI 分析服务 |
| 记录管理 | insert_record, insert_records_batch, query_records, get_record, update_record, delete_record, delete_project_records, get_record_count, check_duplicate | 记录 CRUD |
| Excel 解析 | get_excel_sheets, preview_excel | Excel 预览 |
| 任务管理 | create_processing_task, get_processing_task, list_processing_tasks, update_task_status, create_batch, get_batches | 任务跟踪 |
| 数据处理 | start_processing, pause_processing_task, resume_processing_task, cancel_processing_task | 处理控制 |

## 数据清理机制

根据字段类型自动清理数据：

| 字段类型 | 清理规则 |
|---------|---------|
| phone | 仅保留数字和 + 号 |
| email | 去除空格、换行，转小写 |
| number/id_card | 仅保留数字和字母 |
| date | 仅保留数字和日期分隔符 |
| 其他 | 压缩连续空白为单个空格 |

## 数据库架构

采用 **JSON 统一存储方案**：
- `project_records.data` 以 `field_id` 为 key 存储动态字段值
- 支持 `json_extract()` 进行字段级查询
- 字段改名、调序零成本

## 已知问题修复

### v0.1.0 修复的问题
1. **字段操作导致应用重启** - 添加 `.taurignore` 排除数据库文件监听
2. **结果页面数据不显示** - 修正数据访问方式 `record[field.id]`
3. **进度条卡在"准备中"** - Vue 响应式更新修复
4. **搜索功能缺失** - 实现全文搜索

## 性能指标

| 指标 | Tauri Commands | HTTP API（旧） |
|------|----------------|----------------|
| 通信延迟 | 0ms | ~1-5ms |
| 内存占用 | ~10 MB | ~15-20 MB |
| 启动时间 | ~1 秒 | ~2-3 秒 |
| 架构复杂度 | 简单 | 复杂 |

## 升级说明

本版本为首个正式版本，无需升级。

## 下一步计划

- [ ] 数据导出功能（Excel/CSV）
- [ ] 批量编辑记录
- [ ] 数据统计图表
- [ ] 多语言支持
- [ ] 自动更新功能

---

**感谢使用 reData！**
