# reData - 智能数据处理平台

<div align="center">

**基于 Tauri 构建的智能数据处理桌面应用**

[![Tauri](https://img.shields.io/badge/Tauri-2.x-blue)](https://tauri.app/)
[![Nuxt](https://img.shields.io/badge/Nuxt-4.x-00DC82)](https://nuxt.com/)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.11+-blue)](https://www.python.org/)

</div>

## 📖 项目简介

reData 是一个多项目管理系统，允许用户创建不同的项目，每个项目可以自定义需要提取的字段。系统使用 AI 模型自动识别表头，并从数百万个非标准化的 Excel 文件中提取结构化数据。

### ✨ 核心特性

- 🎯 **多项目管理** - 独立的项目空间，灵活的字段定义
- 🤖 **AI 列映射分析** - 每 Sheet 仅 1 次 AI 调用，节省 99.9% Token
- ✅ **本地验证导入** - 格式规则验证，无需额外 AI 调用
- 🔄 **可配置去重** - 灵活的去重策略（skip/update/merge）
- ⚡ **多文件并行处理** - 实时进度跟踪
- 💾 **本地 SQLite 存储** - 完整数据可追溯
- 🎨 **AI 辅助字段定义** - 自动生成字段元数据

## 🏗️ 技术架构

### 前端
- **Nuxt 4.x** - 全栈 Vue 框架
- **Nuxt UI 4.x** - 基于 Reka UI 和 Tailwind CSS
- **TypeScript** - 完整类型安全
- **Pinia** - 状态管理

### 桌面框架
- **Tauri 2.x** - 轻量级桌面应用框架

### 后端（双实现）

#### Python 后端（生产版本）
- **FastAPI** - 现代 Python Web 框架
- **SQLAlchemy** - Python ORM
- **pandas + openpyxl** - Excel 处理
- **OpenAI SDK** - AI 集成

#### Rust 后端（高性能版本）🚀
- **Axum 0.7** - 高性能异步 Web 框架
- **SeaORM 1.0** - 异步 ORM
- **async-openai 0.24** - OpenAI API 客户端
- **calamine + rust_xlsxwriter** - Excel 处理
- **DDD 架构** - 领域驱动设计

### 数据库
- **SQLite 3.40+** - 本地数据库

## 🚀 快速开始

### 环境要求

- **Node.js** 18+
- **Python** 3.11+ (如果使用 Python 后端)
- **Rust** 1.75+ (如果使用 Rust 后端)
- **uv** (Python 包管理器)

### 安装依赖

```bash
# 克隆仓库
git clone <repository-url>
cd reData

# 安装前端依赖
cd redata-app
npm install

# 安装 Python 后端依赖
cd backend
uv sync
cd ..

# Rust 后端依赖会在构建时自动安装
```

### 开发模式

#### 方式 1：使用 Rust 后端（推荐）🚀

```bash
# 终端 1：启动 Rust 后端
cd redata-app/src-tauri
cargo run --bin server

# 终端 2：启动前端
cd redata-app
npm run dev
```

访问 http://localhost:3000

#### 方式 2：使用 Python 后端

```bash
# 终端 1：启动 Python 后端
cd redata-app/backend
uv run python run.py

# 终端 2：启动前端（需修改配置）
cd redata-app
# 编辑 app/utils/api.ts，设置 USE_RUST_BACKEND = false
npm run dev
```

#### 方式 3：Tauri 开发模式

```bash
cd redata-app
npm run tauri:dev
```

### 生产构建

```bash
cd redata-app
npm run tauri:build
```

## 📊 性能对比

| 指标 | Rust 后端 🚀 | Python 后端 |
|------|-------------|-------------|
| 启动时间 | ~1 秒 | ~2-3 秒 |
| 内存占用 | ~10 MB | ~50 MB |
| API 响应 | < 5ms | ~10-20ms |
| 并发性能 | 优秀 | 良好 |

## 🎯 Rust 后端实现进度

- ✅ **Phase 1**: 基础架构搭建（DDD 架构、错误处理、日志系统）
- ✅ **Phase 2**: 数据库层实现（SeaORM、数据模型、自动迁移、加密工具）
- ✅ **Phase 3**: 项目管理 API（完整 CRUD 操作）
- ⏳ **Phase 4**: 字段管理 API
- ⏳ **Phase 5**: AI 配置管理 API
- ⏳ **Phase 6**: 文件管理 API
- ⏳ **Phase 7**: 数据处理核心
- ⏳ **Phase 8**: 处理任务 API
- ⏳ **Phase 9**: 结果管理 API

## 📚 文档

- [CLAUDE.md](CLAUDE.md) - Claude Code 工作指南
- [RUST_BACKEND_TESTING.md](redata-app/RUST_BACKEND_TESTING.md) - Rust 后端测试指南
- [DDD_ARCHITECTURE.md](redata-app/backend/DDD_ARCHITECTURE.md) - DDD 架构设计文档
- [RUST_MIGRATION_PLAN.md](redata-app/backend/RUST_MIGRATION_PLAN.md) - Rust 迁移计划

## 🔧 API 文档

### Python 后端
- Swagger UI: http://127.0.0.1:8000/docs
- ReDoc: http://127.0.0.1:8000/redoc

### Rust 后端
- 健康检查: http://127.0.0.1:8001/health
- 项目 API: http://127.0.0.1:8001/api/projects

## 🗄️ 数据库

数据库文件位置：`redata-app/backend/data/app.db`

首次运行时自动创建所有表结构。

## 🔐 安全

- API 密钥使用 AES-256-GCM 加密存储
- 数据库文件保持本地，不上传云端
- 参数化查询防止 SQL 注入
- 文件路径验证防止目录遍历

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

[MIT License](LICENSE)

## 🙏 致谢

- [Tauri](https://tauri.app/) - 桌面应用框架
- [Nuxt](https://nuxt.com/) - Vue 全栈框架
- [Axum](https://github.com/tokio-rs/axum) - Rust Web 框架
- [SeaORM](https://www.sea-ql.org/SeaORM/) - Rust ORM

---

**版本**: v2.4.0
**最后更新**: 2026-02-18
