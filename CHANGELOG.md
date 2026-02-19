# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-02-19

### Added

#### Excel 导出功能
- **xlsx 格式导出** - 使用 rust_xlsxwriter 生成 Excel 文件
- **三种导出范围** - 全部数据/筛选结果/当前页
- **原生保存对话框** - 选择导出路径和文件名
- **可选导出内容** - 自选字段、导入时间、来源文件

#### 数据结果页增强
- **每页数量选择** - 支持 50/100/200/500 条每页
- **导入时间列** - 显示记录导入时间
- **高级筛选** - 13 种运算符（等于/包含/范围/为空等），且/或组合

#### 数据处理页优化
- **任务重启复用文件** - 重新开始时自动复用原文件路径
- **进度显示优化** - 修复重复文件名显示问题

#### 项目分组管理
- **分组筛选** - 按分组筛选项目，支持 URL 参数持久化
- **分组树形结构** - 支持父子层级关系
- **批量移动项目** - 将多个项目移动到指定分组

### Changed

#### UI/UX 优化
- **设置页简化** - 危险操作仅保留"删除项目"
- **分组筛选持久化** - 使用 URL 参数保持筛选状态

### Fixed

- 修复分组筛选功能失效
- 修复分组筛选状态在导航后丢失
- 修复 pageSize 修改后数据不加载

### Technical

#### 性能优化
- **N+1 查询优化** - `get_project_batches_with_stats` 改为单次聚合查询
- **批量插入优化** - `insert_records_batch` 使用事务批量插入（500条/批）
- **批量更新优化** - `delete_project_group` 使用单条 UPDATE 语句

#### 安全增强
- **加密密钥校验** - 生产环境强制要求设置 ENCRYPTION_KEY
- **SQL 注入防护** - 筛选字段 ID 验证（仅允许数字）
- **调试日志清理** - 移除生产代码中的 console.log/eprintln

#### Tauri Commands（新增）
- `export_records_xlsx` - 导出记录为 xlsx 文件
- `get_project_batches_with_stats` - 获取项目导入记录（优化版）
- `batch_move_projects` - 批量移动项目到分组
- `move_project_to_group` - 移动单个项目到分组
- `get_project_groups` / `get_project_groups_flat` - 获取分组列表
- `create_project_group` / `update_project_group` / `delete_project_group` - 分组 CRUD
- `reorder_project_groups` - 分组排序

---

## [0.1.1] - 2026-02-19

### Added

#### AI 列映射增强
- **AI 提取要求** - 字段定义支持 `extraction_hint`，允许用户自定义提取提示词
- **动态系统提示** - 根据实际使用的字段类型动态生成 AI 规则表，减少无关信息干扰
- **两步验证机制** - AI 列映射时执行"列名语义 + 数据内容"双重验证，降低误判率
- **字段类型规则表** - 为每种类型添加详细的数据特征描述和常见误判陷阱提示
- **公司字段类型** - 新增 `company` 类型，自动识别公司名称（支持中英文企业标识）

#### 任务管理
- **任务进度持久化** - 新增 `task_file_progress` 表，持久化文件和 Sheet 级别的处理进度
- **任务重置功能** - 已完成/中断的任务支持重新开始，可选是否删除已导入记录

### Changed

#### UI/UX 优化
- **数据处理页面重构** - 优化左右分栏布局，修复滚动问题
- **Sheet 统计信息** - 完成 Sheet 显示成功/失败行数统计
- **深色/浅色模式** - 使用 Nuxt UI 语义化颜色类，优化主题切换体验
- **应用图标更新** - 更新所有平台（Windows/macOS/Linux/iOS/Android）图标
- **系统字体** - 移除外部字体依赖，使用系统字体提升加载速度

#### 代码优化
- **移除主题设置** - 简化设置页面，使用 Nuxt UI 内置主题管理

### Fixed

- 修复原始数据（raw_data）未正确返回的问题
- 修复数据处理相关的响应式更新问题
- 修复任务中断后状态未正确持久化的问题

### Technical

- 数据库迁移：新增 `task_file_progress` 表
- Tauri Commands：更新字段定义结构，新增进度持久化相关函数

---

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

- 批量编辑记录
- 数据统计图表
- 多语言支持
- 自动更新功能

### [0.3.0] - 计划中

- 自定义字段类型
- 数据对比功能

---

[0.1.2]: https://github.com/your-repo/reData/releases/tag/v0.1.2
[0.1.1]: https://github.com/your-repo/reData/releases/tag/v0.1.1
[0.1.0]: https://github.com/your-repo/reData/releases/tag/v0.1.0
