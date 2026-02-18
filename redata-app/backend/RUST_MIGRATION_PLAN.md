# reData Rust 后端重构实施计划

## 一、项目概述

### 1.1 目标

使用 Rust 重新实现 reData 后端，替代现有的 Python + FastAPI 实现，同时保留 Python 版本作为参考和备份。

### 1.2 核心优势

- **性能提升**：Rust 的零成本抽象和高效内存管理
- **类型安全**：编译时类型检查，减少运行时错误
- **并发安全**：Rust 的所有权系统保证线程安全
- **更小的二进制**：单一可执行文件，无需 Python 运行时
- **更好的集成**：与 Tauri 原生集成，减少进程间通信开销

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
| Web 框架 | FastAPI | Axum | 高性能异步 Web 框架 |
| 数据库 ORM | SQLAlchemy | SeaORM | 异步 ORM，支持 SQLite |
| 序列化 | Pydantic | serde | JSON 序列化/反序列化 |
| AI SDK | openai-python | async-openai | OpenAI API 客户端 |
| Excel 处理 | pandas + openpyxl | calamine + rust_xlsxwriter | Excel 读写 |
| WebSocket | FastAPI WebSocket | axum::extract::ws | WebSocket 支持 |
| 异步运行时 | asyncio | tokio | 异步运行时 |

### 2.2 依赖库清单

```toml
[dependencies]
# Web 框架
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

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

# 配置
dotenvy = "0.15"

# 正则表达式
regex = "1"

# WebSocket
tokio-tungstenite = "0.23"

# 加密（用于 API 密钥）
aes-gcm = "0.10"
base64 = "0.22"
```

## 三、架构设计

### 3.1 目录结构

```
redata-app/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # 主入口
│   │   ├── lib.rs               # Tauri 库入口
│   │   └── backend/             # Rust 后端实现
│   │       ├── mod.rs           # 后端模块入口
│   │       ├── api/             # API 路由层
│   │       │   ├── mod.rs
│   │       │   ├── projects.rs
│   │       │   ├── fields.rs
│   │       │   ├── ai_configs.rs
│   │       │   ├── files.rs
│   │       │   ├── processing.rs
│   │       │   └── results.rs
│   │       ├── models/          # 数据模型
│   │       │   ├── mod.rs
│   │       │   ├── project.rs
│   │       │   ├── field.rs
│   │       │   ├── ai_config.rs
│   │       │   ├── task.rs
│   │       │   └── schemas.rs
│   │       ├── services/        # 业务逻辑层
│   │       │   ├── mod.rs
│   │       │   ├── ai_client.rs
│   │       │   ├── validator.rs
│   │       │   ├── excel_parser.rs
│   │       │   ├── extractor.rs
│   │       │   └── storage.rs
│   │       ├── db/              # 数据库层
│   │       │   ├── mod.rs
│   │       │   └── connection.rs
│   │       └── utils/           # 工具函数
│   │           ├── mod.rs
│   │           ├── crypto.rs
│   │           └── error.rs
│   └── Cargo.toml
└── backend/                     # Python 后端（保留）
    └── ...
```

### 3.2 分层架构

```
┌─────────────────────────────────────┐
│         Tauri Frontend              │
│      (Nuxt 4 + TypeScript)          │
└─────────────────────────────────────┘
                 │
                 │ HTTP/WebSocket
                 ↓
┌─────────────────────────────────────┐
│         API Layer (Axum)            │
│  - 路由定义                          │
│  - 请求验证                          │
│  - 响应序列化                        │
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
│      Database Layer (SeaORM)       │
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

### Phase 1: 基础架构搭建 (Week 1)

**目标**: 建立 Rust 后端的基础框架

#### Todolist

- [ ] 1.1 创建 Rust 后端模块结构
  - [ ] 在 `src-tauri/src/` 下创建 `backend/` 目录
  - [ ] 创建各子模块的 `mod.rs` 文件
  - [ ] 配置模块导出

- [ ] 1.2 配置 Cargo.toml 依赖
  - [ ] 添加 Axum Web 框架
  - [ ] 添加 SeaORM 数据库 ORM
  - [ ] 添加 tokio 异步运行时
  - [ ] 添加 serde 序列化库
  - [ ] 添加其他核心依赖

- [ ] 1.3 实现基础错误处理
  - [ ] 定义 `AppError` 类型
  - [ ] 实现 `IntoResponse` trait
  - [ ] 创建错误转换函数

- [ ] 1.4 实现日志系统
  - [ ] 配置 tracing-subscriber
  - [ ] 添加日志中间件
  - [ ] 设置日志级别

- [ ] 1.5 创建 Axum 服务器
  - [ ] 实现基础路由
  - [ ] 配置 CORS
  - [ ] 添加健康检查端点 `GET /health`

### Phase 2: 数据库层实现 (Week 1-2)

**目标**: 实现数据库连接和 ORM 模型

#### Todolist

- [ ] 2.1 配置 SeaORM
  - [ ] 创建数据库连接池
  - [ ] 实现数据库初始化函数
  - [ ] 配置 SQLite 连接参数

- [ ] 2.2 定义数据模型 (Entity)
  - [ ] `Project` 模型
  - [ ] `ProjectField` 模型
  - [ ] `AiConfig` 模型
  - [ ] `ProcessingTask` 模型
  - [ ] 动态项目数据表模型

- [ ] 2.3 实现数据库迁移
  - [ ] 创建表结构
  - [ ] 添加索引
  - [ ] 实现自动迁移

- [ ] 2.4 实现加密工具
  - [ ] API 密钥加密函数
  - [ ] API 密钥解密函数
  - [ ] 密钥管理

### Phase 3: 项目管理 API (Week 2)

**目标**: 实现项目 CRUD 操作

#### Todolist

- [ ] 3.1 定义 API Schemas
  - [ ] `ProjectCreate` 请求结构
  - [ ] `ProjectUpdate` 请求结构
  - [ ] `ProjectResponse` 响应结构

- [ ] 3.2 实现项目 API 路由
  - [ ] `GET /api/projects/` - 查询项目列表
  - [ ] `POST /api/projects/` - 新建项目
  - [ ] `GET /api/projects/{id}` - 查询项目详情
  - [ ] `PUT /api/projects/{id}` - 更新项目
  - [ ] `DELETE /api/projects/{id}` - 删除项目

- [ ] 3.3 实现项目服务层
  - [ ] 创建项目时自动创建数据表
  - [ ] 删除项目时级联删除
  - [ ] 项目去重配置管理

### Phase 4: 字段管理 API (Week 2-3)

**目标**: 实现字段定义和管理

#### Todolist

- [ ] 4.1 定义字段 API Schemas
  - [ ] `FieldCreate` 请求结构
  - [ ] `FieldUpdate` 请求结构
  - [ ] `FieldResponse` 响应结构

- [ ] 4.2 实现字段 API 路由
  - [ ] `POST /api/fields/` - 添加字段
  - [ ] `PUT /api/fields/{id}` - 编辑字段
  - [ ] `DELETE /api/fields/{id}` - 软删除字段
  - [ ] `POST /api/fields/{id}/restore` - 恢复字段
  - [ ] `GET /api/fields/project/{id}` - 查询字段列表
  - [ ] `GET /api/fields/project/{id}/all` - 查询全部字段

- [ ] 4.3 实现动态表结构管理
  - [ ] 添加字段时 ALTER TABLE
  - [ ] 删除字段时标记 is_deleted
  - [ ] 智能表结构迁移

- [ ] 4.4 实现 AI 辅助字段生成
  - [ ] `POST /api/fields/generate-metadata` - AI 生成字段元数据

### Phase 5: AI 配置管理 API (Week 3)

**目标**: 实现 AI 配置的 CRUD 和测试

#### Todolist

- [ ] 5.1 定义 AI 配置 Schemas
  - [ ] `AiConfigCreate` 请求结构
  - [ ] `AiConfigUpdate` 请求结构
  - [ ] `AiConfigResponse` 响应结构

- [ ] 5.2 实现 AI 配置 API 路由
  - [ ] `POST /api/ai-configs/` - 新增配置
  - [ ] `GET /api/ai-configs/` - 查询配置列表
  - [ ] `GET /api/ai-configs/{id}` - 查询单个配置
  - [ ] `GET /api/ai-configs/default` - 查询默认配置
  - [ ] `PUT /api/ai-configs/{id}` - 更新配置
  - [ ] `DELETE /api/ai-configs/{id}` - 删除配置
  - [ ] `POST /api/ai-configs/test-connection` - 测试连接
  - [ ] `POST /api/ai-configs/{id}/set-default` - 设置默认

- [ ] 5.3 实现 AI 客户端服务
  - [ ] 使用 async-openai 库
  - [ ] 支持自定义 API URL
  - [ ] 实现连接测试

### Phase 6: 文件管理 API (Week 3-4)

**目标**: 实现文件上传、预览和管理

#### Todolist

- [ ] 6.1 实现文件上传
  - [ ] `POST /api/files/upload` - 单文件上传
  - [ ] `POST /api/files/upload-multiple` - 批量上传
  - [ ] 文件存储到临时目录

- [ ] 6.2 实现文件预览
  - [ ] `GET /api/files/preview/{id}` - 预览前 10 行
  - [ ] `GET /api/files/info/{id}` - 获取文件信息
  - [ ] 支持多 Sheet 预览

- [ ] 6.3 实现文件管理
  - [ ] `DELETE /api/files/{id}` - 删除文件
  - [ ] `GET /api/files/batch/{batch}` - 获取批次文件
  - [ ] `GET /api/files/download/{id}` - 下载文件
  - [ ] `POST /api/files/cleanup` - 清理临时文件

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

### Phase 8: 处理任务 API (Week 5-6)

**目标**: 实现任务管理和进度跟踪

#### Todolist

- [ ] 8.1 实现任务 API 路由
  - [ ] `POST /api/processing/start` - 启动任务
  - [ ] `POST /api/processing/pause/{id}` - 暂停任务
  - [ ] `POST /api/processing/resume/{id}` - 恢复任务
  - [ ] `POST /api/processing/cancel/{id}` - 取消任务
  - [ ] `GET /api/processing/status/{id}` - 查询状态
  - [ ] `GET /api/processing/list/{project_id}` - 任务列表

- [ ] 8.2 实现 WebSocket 进度推送
  - [ ] `WS /api/processing/ws/progress/{id}` - 实时进度
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

### Phase 9: 结果管理 API (Week 6)

**目标**: 实现数据查询、编辑和导出

#### Todolist

- [ ] 9.1 实现结果查询 API
  - [ ] `GET /api/results/{project_id}` - 结果列表
  - [ ] `GET /api/results/{project_id}/{record_id}` - 单条记录
  - [ ] 分页支持
  - [ ] 批次筛选
  - [ ] 状态筛选
  - [ ] 关键词搜索
  - [ ] 排序

- [ ] 9.2 实现结果编辑 API
  - [ ] `PUT /api/results/{project_id}/{record_id}` - 编辑记录
  - [ ] `DELETE /api/results/{project_id}/{record_id}` - 删除记录
  - [ ] `DELETE /api/results/{project_id}/batch/{batch}` - 删除批次

- [ ] 9.3 实现结果导出 API
  - [ ] `GET /api/results/export/{project_id}` - 导出结果
  - [ ] 支持 xlsx 格式
  - [ ] 支持 csv 格式
  - [ ] 批次筛选

- [ ] 9.4 实现统计 API
  - [ ] `GET /api/results/statistics/{project_id}` - 项目统计
  - [ ] 总记录数
  - [ ] 成功/失败数
  - [ ] 批次数

### Phase 10: 集成和测试 (Week 7)

**目标**: 集成到 Tauri，测试和优化

#### Todolist

- [ ] 10.1 Tauri 集成
  - [ ] 修改 `lib.rs` 启动 Rust 后端
  - [ ] 移除 Python 进程管理代码
  - [ ] 配置端口和路径

- [ ] 10.2 前端适配
  - [ ] 确保 API 兼容性
  - [ ] 测试所有功能
  - [ ] 修复兼容性问题

- [ ] 10.3 性能优化
  - [ ] 数据库查询优化
  - [ ] 并发处理优化
  - [ ] 内存使用优化

- [ ] 10.4 错误处理完善
  - [ ] 统一错误响应格式
  - [ ] 添加详细错误信息
  - [ ] 日志记录

- [ ] 10.5 文档编写
  - [ ] API 文档
  - [ ] 部署文档
  - [ ] 迁移指南

## 五、与 Python 版本共存策略

### 5.1 开发阶段

- Python 后端保留在 `backend/` 目录
- Rust 后端实现在 `src-tauri/src/backend/` 目录
- 使用不同的端口（Python: 8000, Rust: 8001）
- 前端可以通过环境变量切换后端

### 5.2 测试阶段

- 并行运行两个后端
- 对比功能和性能
- 逐步迁移前端调用

### 5.3 生产阶段

- Rust 后端稳定后，默认使用 Rust
- Python 后端作为备份保留
- 提供切换机制

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

- [ ] 所有 60+ 个功能点实现
- [ ] API 接口 100% 兼容
- [ ] 前端无需修改即可使用

### 7.2 性能指标

- [ ] API 响应时间 < 100ms (P95)
- [ ] Excel 处理速度 > 1000 行/秒
- [ ] 内存占用 < 100MB (空闲)
- [ ] 二进制大小 < 50MB

### 7.3 质量标准

- [ ] 单元测试覆盖率 > 80%
- [ ] 集成测试通过率 100%
- [ ] 无内存泄漏
- [ ] 无数据丢失

## 八、下一步行动

1. **立即开始**: Phase 1 - 基础架构搭建
2. **第一个里程碑**: 完成 Phase 3 - 项目管理 API
3. **第二个里程碑**: 完成 Phase 7 - 数据处理核心
4. **最终目标**: 完成所有 10 个阶段，替代 Python 后端

---

**文档版本**: v1.0  
**创建日期**: 2026-02-18  
**最后更新**: 2026-02-18  
**负责人**: reData Team
