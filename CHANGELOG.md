# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-18

### Added

#### 核心功能
- **多项目管理** - 创建、编辑、删除项目，项目级别数据隔离
- **字段定义** - 类 Excel 表格编辑器，支持 text、phone、email、number、date、id_card 类型
- **AI 列映射分析** - 每 Sheet 仅 1 次 AI 调用，智能识别表头和列映射
- **本地验证导入** - 格式验证（正则表达式）+ 智能数据清理
- **智能去重** - 可配置去重字段和策略
- **数据搜索** - 全文搜索 JSON 数据，300ms 防抖优化
- **实时进度推送** - Tauri 事件系统，零延迟通信
- **任务控制** - 暂停、恢复、取消处理任务

#### 数据清理机制
- phone: 仅保留数字和 + 号
- email: 去除空格、换行，转小写
- number/id_card: 仅保留数字和字母
- date: 仅保留数字和日期分隔符
- 其他: 压缩连续空白为单个空格

#### Tauri Commands（36 个）
- 项目管理: get_projects, get_project, create_project, update_project, delete_project
- 字段管理: get_fields, get_all_fields, create_field, update_field, delete_field, restore_field, generate_field_metadata
- AI 配置: get_ai_configs, get_ai_config, get_default_ai_config, create_ai_config, update_ai_config, delete_ai_config, set_default_ai_config, test_ai_connection
- AI 服务: analyze_column_mapping, ai_generate_field_metadata
- 记录管理: insert_record, insert_records_batch, query_records, get_record, update_record, delete_record, delete_project_records, get_record_count, check_duplicate
- Excel 解析: get_excel_sheets, preview_excel
- 任务管理: create_processing_task, get_processing_task, list_processing_tasks, update_task_status, create_batch, get_batches
- 数据处理: start_processing, pause_processing_task, resume_processing_task, cancel_processing_task

### Technical

#### 架构
- **Tauri Commands 模式** - 零网络开销的前后端通信
- **Tauri 事件系统** - 替代 WebSocket 进行实时进度推送
- **JSON 统一存储** - 以 field_id 为 key 存储动态字段值
- **DDD 架构** - Rust 后端采用领域驱动设计

#### 技术栈
- 前端: Nuxt 4.x + Nuxt UI 4.x + TypeScript + Pinia
- 桌面框架: Tauri 2.x
- 后端: Rust + SeaORM 1.0 + async-openai 0.24 + calamine
- 数据库: SQLite 3.40+

#### 性能
- 通信延迟: 0ms（直接函数调用）
- 内存占用: ~10 MB
- 启动时间: ~1 秒

### Fixed

- 字段操作导致应用重启 - 添加 `.taurignore` 排除数据库文件监听
- 结果页面数据不显示 - 修正数据访问方式 `record[field.id]`
- 进度条卡在"准备中" - Vue 响应式更新修复
- 搜索功能缺失 - 实现全文搜索

### Removed

- Python FastAPI 后端（已完全迁移到 Rust）

---

## 版本规划

### [0.2.0] - 计划中

- 数据导出功能（Excel/CSV）
- 批量编辑记录
- 数据统计图表
- 多语言支持
- 自动更新功能

### [0.3.0] - 计划中

- 自定义字段类型
- 高级筛选功能
- 数据对比功能
- 批次管理优化

---

[0.1.0]: https://github.com/your-repo/reData/releases/tag/v0.1.0
