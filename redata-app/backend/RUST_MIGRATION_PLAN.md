# reData Rust 后端重构实施计划

## ⚠️ 架构变更说明（v2.5.0）

**重要更新**：项目已从原计划的 HTTP API 架构改为 **Tauri Commands 架构**。

**变更原因**：
- 零网络开销：直接函数调用，无 HTTP 请求
- 更简单的架构：无需管理 HTTP 服务器生命周期
- 更好的性能：通信延迟从 1-5ms 降低到 0ms
- 更强的类型安全：Rust + TypeScript 完全类型安全

**影响**：
- API 路由层改为 Commands 层（`src-tauri/src/commands/`）
- 前端使用 `invoke()` 而不是 HTTP 请求
- 移除 Axum HTTP 服务器相关代码
- 保留核心业务逻辑（services、models、database）

**当前进度**：
- ✅ Phase 1: 基础架构搭建（已完成）
- ✅ Phase 2: 数据库层实现（已完成）
- ✅ Phase 3: 项目管理（已完成，使用 Tauri Commands）
- ⏳ Phase 4-9: 待迁移到 Tauri Commands

## 一、项目概述

### 1.1 目标

使用 Rust 重新实现 reData 后端，替代现有的 Python + FastAPI 实现，同时保留 Python 版本作为参考和备份。

### 1.2 核心优势

- **零网络开销**：Tauri Commands 直接函数调用，无 HTTP 请求
- **性能提升**：Rust 的零成本抽象和高效内存管理
- **类型安全**：编译时类型检查，减少运行时错误
- **并发安全**：Rust 的所有权系统保证线程安全
- **更小的二进制**：单一可执行文件，无需 Python 运行时
- **架构简化**：无需管理 HTTP 服务器，降低复杂度

### 1.3 功能范围

参考 `TODOLIST.md`，共 60+ 个功能点：
- 项目管理（10 个功能）
- 字段管理（7 个功能）
- AI 配置管理（8 个功能）
- 文件管理（8 个功能）
- 数据处理（14 个功能）
- 结果管理（7 个功能）
- 基础设施（6 个功能）

## 二、技术栈选择

### 2.1 核心框架

| 组件 | Python 版本 | Rust 版本 | 说明 |
|------|------------|-----------|------|
| 通信模式 | HTTP API | Tauri Commands | 零网络开销的直接调用 |
| 数据库 ORM | SQLAlchemy | SeaORM | 异步 ORM，支持 SQLite |
| 序列化 | Pydantic | serde | JSON 序列化/反序列化 |
| AI SDK | openai-python | async-openai | OpenAI API 客户端 |
| Excel 处理 | pandas + openpyxl | calamine + rust_xlsxwriter | Excel 读写 |
| 实时更新 | WebSocket | WebSocket（待实现） | 进度推送 |
| 异步运行时 | asyncio | tokio | 异步运行时 |

### 2.2 依赖库清单

```toml
[dependencies]
# Tauri 框架
tauri = { version = "2", features = ["..."] }

# 异步运行时
tokio = { version = "1", features = ["full"] }

# 数据库
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio-native-tls"] }

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# AI 集成
async-openai = "0.24"

# Excel 处理
calamine = "0.26"
rust_xlsxwriter = "0.76"

# 日期时间
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1", features = ["v4", "serde"] }

# 错误处理
anyhow = "1"
thiserror = "1"

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 正则表达式
regex = "1"

# 加密（用于 API 密钥）
aes-gcm = "0.10"
base64 = "0.22"
```

**注意**：不再需要 Axum、tower、tower-http 等 HTTP 服务器相关依赖。

## 三、架构设计

### 3.1 目录结构

```
redata-app/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # Tauri 主入口
│   │   ├── lib.rs               # Tauri 库入口（注册 Commands）
│   │   ├── commands/            # Tauri Commands 层 ✅
│   │   │   ├── mod.rs
│   │   │   ├── projects.rs      # 项目管理 Commands ✅
│   │   │   ├── fields.rs        # 字段管理 Commands
│   │   │   ├── ai_configs.rs    # AI 配置 Commands
│   │   │   ├── files.rs         # 文件管理 Commands
│   │   │   ├── processing.rs    # 数据处理 Commands
│   │   │   └── results.rs       # 结果管理 Commands
│   │   └── backend/             # Rust 后端核心逻辑
│   │       ├── mod.rs           # 后端模块入口
│   │       ├── domain/          # 领域层（DDD）
│   │       ├── application/     # 应用层（DDD）
│   │       ├── infrastructure/  # 基础设施层（DDD）✅
│   │       │   ├── persistence/ # 数据库层 ✅
│   │       │   │   ├── database.rs      ✅
│   │       │   │   ├── migrations.rs    ✅
│   │       │   │   ├── models/          ✅
│   │       │   │   └── repositories/
│   │       │   └── config/      # 配置层 ✅
│   │       │       ├── error.rs         ✅
│   │       │       ├── logging.rs       ✅
│   │       │       └── crypto.rs        ✅
│   │       ├── presentation/    # 表现层（已弃用 HTTP API）
│   │       └── services/        # 业务逻辑层
│   │           ├── mod.rs
│   │           ├── ai_client.rs
│   │           ├── validator.rs
│   │           ├── excel_parser.rs
│   │           ├── extractor.rs
│   │           └── storage.rs
│   └── Cargo.toml
└── backend/                     # Python 后端（保留作为参考）
    └── ...
```

### 3.2 分层架构

```
┌─────────────────────────────────────┐
│         Tauri Frontend              │
│      (Nuxt 4 + TypeScript)          │
└─────────────────────────────────────┘
                 │
                 │ invoke()
                 ↓
┌─────────────────────────────────────┐
│      Commands Layer (Tauri)         │
│  - 项目管理 Commands ✅               │
│  - 字段管理 Commands                 │
│  - AI 配置 Commands                  │
│  - 文件管理 Commands                 │
│  - 数据处理 Commands                 │
│  - 结果管理 Commands                 │
└─────────────────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────┐
│       Service Layer                 │
│  - 业务逻辑                          │
│  - AI 集成                           │
│  - Excel 处理                        │
│  - 数据验证                          │
└─────────────────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────┐
│      Database Layer (SeaORM) ✅     │
│  - ORM 映射                          │
│  - 查询构建                          │
│  - 事务管理                          │
└─────────────────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────┐
│         SQLite Database             │
└─────────────────────────────────────┘
```

## 四、分阶段实施计划

### Phase 1: 基础架构搭建 (Week 1) ✅

**目标**: 建立 Rust 后端的基础框架

**状态**: 已完成

#### Todolist

- [x] 1.1 创建 Rust 后端模块结构
  - [x] 在 `src-tauri/src/` 下创建 `backend/` 目录
  - [x] 创建各子模块的 `mod.rs` 文件
  - [x] 配置模块导出

- [x] 1.2 配置 Cargo.toml 依赖
  - [x] 添加 SeaORM 数据库 ORM
  - [x] 添加 tokio 异步运行时
  - [x] 添加 serde 序列化库
  - [x] 添加其他核心依赖

- [x] 1.3 实现基础错误处理
  - [x] 定义 `AppError` 类型
  - [x] 实现 `IntoResponse` trait
  - [x] 创建错误转换函数

- [x] 1.4 实现日志系统
  - [x] 配置 tracing-subscriber
  - [x] 添加日志中间件
  - [x] 设置日志级别

- [x] 1.5 创建 Commands 模块
  - [x] 创建 `commands/` 目录
  - [x] 实现基础 Commands 结构
  - [x] 在 lib.rs 中注册 Commands

### Phase 2: 数据库层实现 (Week 1-2) ✅

**目标**: 实现数据库连接和 ORM 模型

**状态**: 已完成

#### Todolist

- [x] 2.1 配置 SeaORM
  - [x] 创建数据库连接池
  - [x] 实现数据库初始化函数
  - [x] 配置 SQLite 连接参数

- [x] 2.2 定义数据模型 (Entity)
  - [x] `Project` 模型
  - [x] `ProjectField` 模型
  - [x] `AiConfig` 模型
  - [x] `ProcessingTask` 模型
  - [x] `Batch` 模型
  - [ ] 动态项目数据表模型（待实现）

- [x] 2.3 实现数据库迁移
  - [x] 创建表结构
  - [x] 添加索引
  - [x] 实现自动迁移

- [x] 2.4 实现加密工具
  - [x] API 密钥加密函数
  - [x] API 密钥解密函数
  - [x] 密钥管理

### Phase 3: 项目管理 Commands (Week 2) ✅

**目标**: 实现项目 CRUD 操作（使用 Tauri Commands）

**状态**: 已完成

#### Todolist

- [x] 3.1 定义 Commands Schemas
  - [x] `CreateProjectRequest` 请求结构
  - [x] `UpdateProjectRequest` 请求结构
  - [x] `ProjectResponse` 响应结构

- [x] 3.2 实现项目 Commands
  - [x] `get_projects` - 查询项目列表
  - [x] `create_project` - 新建项目
  - [x] `get_project` - 查询项目详情
  - [x] `update_project` - 更新项目
  - [x] `delete_project` - 删除项目

- [x] 3.3 实现项目服务层
  - [x] 创建项目时自动创建数据表
  - [x] 删除项目时级联删除
  - [x] 项目去重配置管理

- [x] 3.4 前端集成
  - [x] 更新 API 客户端使用 invoke()
  - [x] 测试所有功能

### Phase 4: 字段管理 Commands (Week 2-3)

**目标**: 实现字段定义和管理（使用 Tauri Commands）

#### Todolist

- [ ] 4.1 定义字段 Commands Schemas
  - [ ] `FieldCreate` 请求结构
  - [ ] `FieldUpdate` 请求结构
  - [ ] `FieldResponse` 响应结构

- [ ] 4.2 实现字段 Commands
  - [ ] `create_field` - 添加字段
  - [ ] `update_field` - 编辑字段
  - [ ] `delete_field` - 软删除字段
  - [ ] `restore_field` - 恢复字段
  - [ ] `get_fields` - 查询字段列表
  - [ ] `get_all_fields` - 查询全部字段（包括已删除）

- [ ] 4.3 实现动态表结构管理
  - [ ] 添加字段时 ALTER TABLE
  - [ ] 删除字段时标记 is_deleted
  - [ ] 智能表结构迁移

- [ ] 4.4 实现 AI 辅助字段生成
  - [ ] `generate_field_metadata` - AI 生成字段元数据

### Phase 5: AI 配置管理 Commands (Week 3)

**目标**: 实现 AI 配置的 CRUD 和测试（使用 Tauri Commands）

#### Todolist

- [ ] 5.1 定义 AI 配置 Schemas
  - [ ] `AiConfigCreate` 请求结构
  - [ ] `AiConfigUpdate` 请求结构
  - [ ] `AiConfigResponse` 响应结构

- [ ] 5.2 实现 AI 配置 Commands
  - [ ] `create_ai_config` - 新增配置
  - [ ] `get_ai_configs` - 查询配置列表
  - [ ] `get_ai_config` - 查询单个配置
  - [ ] `get_default_ai_config` - 查询默认配置
  - [ ] `update_ai_config` - 更新配置
  - [ ] `delete_ai_config` - 删除配置
  - [ ] `test_ai_connection` - 测试连接
  - [ ] `set_default_ai_config` - 设置默认

- [ ] 5.3 实现 AI 客户端服务
  - [ ] 使用 async-openai 库
  - [ ] 支持自定义 API URL
  - [ ] 实现连接测试

### Phase 6: 文件管理 Commands (Week 3-4)

**目标**: 实现文件上传、预览和管理（使用 Tauri Commands）

#### Todolist

- [ ] 6.1 实现文件上传
  - [ ] `upload_file` - 单文件上传
  - [ ] `upload_multiple_files` - 批量上传
  - [ ] 文件存储到临时目录

- [ ] 6.2 实现文件预览
  - [ ] `preview_file` - 预览前 10 行
  - [ ] `get_file_info` - 获取文件信息
  - [ ] 支持多 Sheet 预览

- [ ] 6.3 实现文件管理
  - [ ] `delete_file` - 删除文件
  - [ ] `get_batch_files` - 获取批次文件
  - [ ] `download_file` - 下载文件
  - [ ] `cleanup_temp_files` - 清理临时文件

- [ ] 6.4 实现 Excel 解析服务
  - [ ] 使用 calamine 读取 Excel
  - [ ] 支持 .xlsx 和 .xls 格式
  - [ ] 多 Sheet 处理

### Phase 7: 数据处理核心 (Week 4-5)

**目标**: 实现两阶段数据处理流程

#### Todolist

- [ ] 7.1 实现数据验证器
  - [ ] 手机号验证
  - [ ] 邮箱验证
  - [ ] URL 验证
  - [ ] 日期验证
  - [ ] 数字验证
  - [ ] 数据标准化

- [ ] 7.2 实现 AI 列映射分析
  - [ ] 读取前 10 行样本
  - [ ] AI 识别表头位置
  - [ ] AI 分析列映射关系
  - [ ] 返回映射结果和置信度

- [ ] 7.3 实现本地验证导入
  - [ ] 根据映射读取列数据
  - [ ] 格式验证
  - [ ] 逐行插入数据库
  - [ ] 错误记录

- [ ] 7.4 实现去重处理
  - [ ] skip 策略 - 跳过重复
  - [ ] update 策略 - 更新记录
  - [ ] merge 策略 - 合并数据

- [ ] 7.5 实现空行检测
  - [ ] 连续空行计数
  - [ ] 10 行空行后跳到下一 Sheet

- [ ] 7.6 实现多 Sheet 处理
  - [ ] 每个 Sheet 独立表头识别
  - [ ] Sheet 名称记录
  - [ ] 按顺序处理所有 Sheet

### Phase 8: 处理任务 Commands (Week 5-6)

**目标**: 实现任务管理和进度跟踪（使用 Tauri Commands）

#### Todolist

- [ ] 8.1 实现任务 Commands
  - [ ] `start_processing` - 启动任务
  - [ ] `pause_processing` - 暂停任务
  - [ ] `resume_processing` - 恢复任务
  - [ ] `cancel_processing` - 取消任务
  - [ ] `get_processing_status` - 查询状态
  - [ ] `get_processing_list` - 任务列表

- [ ] 8.2 实现 WebSocket 进度推送（待定）
  - [ ] 实时进度推送
  - [ ] 广播进度事件
  - [ ] 连接管理

- [ ] 8.3 实现任务协调器
  - [ ] 异步任务执行
  - [ ] 暂停/恢复机制
  - [ ] 取消机制
  - [ ] 进度计算

- [ ] 8.4 实现批次管理
  - [ ] 批次号生成
  - [ ] 文件复制到 history/
  - [ ] 批次统计

### Phase 9: 结果管理 Commands (Week 6)

**目标**: 实现数据查询、编辑和导出（使用 Tauri Commands）

#### Todolist

- [ ] 9.1 实现结果查询 Commands
  - [ ] `get_results` - 结果列表
  - [ ] `get_result` - 单条记录
  - [ ] 分页支持
  - [ ] 批次筛选
  - [ ] 状态筛选
  - [ ] 关键词搜索
  - [ ] 排序

- [ ] 9.2 实现结果编辑 Commands
  - [ ] `update_result` - 编辑记录
  - [ ] `delete_result` - 删除记录
  - [ ] `delete_batch_results` - 删除批次

- [ ] 9.3 实现结果导出 Commands
  - [ ] `export_results` - 导出结果
  - [ ] 支持 xlsx 格式
  - [ ] 支持 csv 格式
  - [ ] 批次筛选

- [ ] 9.4 实现统计 Commands
  - [ ] `get_statistics` - 项目统计
  - [ ] 总记录数
  - [ ] 成功/失败数
  - [ ] 批次数

### Phase 10: 集成和测试 (Week 7)

**目标**: 完善 Tauri 集成，测试和优化

#### Todolist

- [x] 10.1 Tauri 集成
  - [x] 修改 `lib.rs` 初始化数据库
  - [x] 注册所有 Commands
  - [x] 移除 HTTP 服务器代码

- [x] 10.2 前端适配
  - [x] 更新 API 客户端使用 invoke()
  - [x] 测试项目管理功能
  - [ ] 测试其他功能模块

- [ ] 10.3 性能优化
  - [ ] 数据库查询优化
  - [ ] 并发处理优化
  - [ ] 内存使用优化

- [ ] 10.4 错误处理完善
  - [ ] 统一错误响应格式
  - [ ] 添加详细错误信息
  - [ ] 日志记录

- [x] 10.5 文档编写
  - [x] 更新 CLAUDE.md
  - [x] 更新 README.md
  - [x] 更新测试指南
  - [ ] 编写迁移指南

## 五、与 Python 版本共存策略

### 5.1 开发阶段（当前）

- Python 后端保留在 `backend/` 目录作为参考实现
- Rust 后端实现在 `src-tauri/src/` 目录
- 使用 Tauri Commands 模式（无需端口配置）
- 前端默认使用 Rust 后端（Tauri Commands）

### 5.2 测试阶段

- Rust 后端作为主要实现
- Python 后端作为功能参考
- 对比功能完整性
- 逐步迁移所有功能到 Rust

### 5.3 生产阶段

- Rust 后端作为唯一实现
- Python 后端保留作为文档和参考
- 提供完整的功能覆盖
- 性能和稳定性优于 Python 版本

## 六、风险和挑战

### 6.1 技术风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| SeaORM 学习曲线 | 中 | 参考官方文档，从简单查询开始 |
| Excel 处理复杂度 | 高 | 使用成熟的 calamine 库 |
| WebSocket 实现 | 中 | 使用 Axum 内置 WebSocket 支持 |
| AI SDK 兼容性 | 中 | async-openai 与 Python SDK 接口类似 |
| 动态表结构 | 高 | 使用原生 SQL，避免 ORM 限制 |

### 6.2 时间风险

- 预计总开发时间: 7 周
- 关键路径: Phase 7 (数据处理核心)
- 缓冲时间: 1-2 周

### 6.3 兼容性风险

- 确保 API 接口完全兼容
- 数据库结构保持一致
- 前端无需修改

## 七、成功标准

### 7.1 功能完整性

- [x] Phase 1-3 完成（基础架构、数据库、项目管理）
- [ ] 所有 60+ 个功能点实现
- [ ] Commands 接口完全覆盖
- [ ] 前端无需修改即可使用

### 7.2 性能指标

- [x] 通信延迟 = 0ms（Tauri Commands）
- [x] 启动时间 < 2秒
- [ ] Excel 处理速度 > 1000 行/秒
- [x] 内存占用 < 20MB (空闲)
- [ ] 二进制大小 < 50MB

### 7.3 质量标准

- [ ] 单元测试覆盖率 > 80%
- [ ] 集成测试通过率 100%
- [ ] 无内存泄漏
- [ ] 无数据丢失

## 八、下一步行动

1. **已完成**: ✅ Phase 1-3 - 基础架构、数据库层、项目管理 Commands
2. **当前重点**: Phase 4 - 字段管理 Commands
3. **第二个里程碑**: Phase 7 - 数据处理核心
4. **最终目标**: 完成所有 10 个阶段，全面替代 Python 后端

---

**文档版本**: v2.0
**创建日期**: 2026-02-18
**最后更新**: 2026-02-18
**架构**: Tauri Commands（v2.5.0）
**负责人**: reData Team
