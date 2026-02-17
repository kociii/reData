# reData - 智能数据处理平台

一个基于 Tauri 2 + Nuxt 4 + FastAPI 构建的智能表格数据提取系统桌面应用。

## 项目概述

**reData** 是一个多项目管理的智能数据处理平台，使用 AI 模型自动识别表头，并从数百万个非标准化的 Excel 文件中提取结构化数据。

### 核心能力

- **多项目管理** - 创建不同的项目，每个项目独立管理数据和配置
- **灵活的字段定义** - AI 辅助生成英文字段名、验证规则、提取提示
- **AI 驱动提取** - 每 Sheet 仅 1 次 AI 调用，节省 99.9% Token 消耗
- **全局标签页** - 浏览器式标签页体验，项目列表固定为首标签
- **可配置去重** - 每个项目可以设置是否去重，以及按哪些字段去重
- **多文件并行处理** - 实时进度跟踪
- **本地 SQLite 存储** - 每个项目独立存储，完整数据可追溯

## 技术栈

### 前端
- **Nuxt 4.x** - 最新版全栈 Vue 框架
- **Vue 3.5+** - 渐进式 JavaScript 框架
- **Nuxt UI 4.x** - 基于 Reka UI 和 Tailwind CSS 的 UI 库
- **Pinia** - Vue 状态管理
- **TypeScript** - 类型安全

### 桌面框架
- **Tauri 2.x** - 轻量级桌面应用框架

### 后端
- **FastAPI** - 现代 Python Web 框架
- **SQLAlchemy** - Python ORM
- **OpenAI SDK** - AI 集成
- **pandas + openpyxl** - Excel 处理
- **uvicorn** - ASGI 服务器

### 数据库
- **SQLite 3.40+** - 本地数据库

### 包管理
- **npm** - 前端包管理
- **uv** - Python 包管理

## 项目结构

```
redata-app/
├── app/                    # Nuxt 应用根组件
│   ├── pages/              # 页面组件（自动路由）
│   │   ├── index.vue       # 项目列表页（首页）
│   │   ├── project/        # 项目相关页面
│   │   │   ├── [id].vue    # 项目详情页
│   │   │   └── [id]/       # 项目子页面
│   │   │       ├── fields.vue      # 字段定义
│   │   │       ├── processing.vue  # 数据处理
│   │   │       ├── results.vue     # 结果展示
│   │   │       └── settings.vue    # 项目设置
│   │   └── settings.vue    # 设置页
│   ├── stores/             # Pinia stores
│   │   ├── project.ts      # 项目状态
│   │   ├── field.ts        # 字段状态
│   │   ├── processing.ts   # 处理状态
│   │   ├── result.ts       # 结果状态
│   │   ├── config.ts       # 配置状态
│   │   └── tab.ts          # 标签页状态
│   ├── components/         # 可复用组件
│   ├── types/              # TypeScript 类型定义
│   └── utils/              # 工具函数（API 客户端）
├── backend/                # Python 后端
│   ├── src/redata/
│   │   ├── api/            # API 路由
│   │   │   ├── projects.py
│   │   │   ├── fields.py
│   │   │   ├── files.py
│   │   │   ├── processing.py
│   │   │   ├── results.py
│   │   │   └── ai_configs.py
│   │   ├── db/             # 数据库配置
│   │   ├── models/         # 数据模型
│   │   ├── services/       # 业务服务
│   │   │   ├── ai_client.py    # AI 客户端
│   │   │   ├── excel_parser.py # Excel 解析
│   │   │   ├── extractor.py    # 数据提取
│   │   │   ├── storage.py      # 数据存储
│   │   │   └── validator.py    # 数据验证
│   │   └── main.py         # FastAPI 应用
│   ├── pyproject.toml      # Python 依赖
│   └── run.py              # 启动脚本
├── src-tauri/              # Tauri 配置
│   ├── tauri.conf.json
│   └── src/
├── nuxt.config.ts          # Nuxt 配置
└── package.json            # Node 依赖
```

## 开发环境要求

- **Node.js** 18.0+
- **Python** 3.11+
- **Rust** 1.75+ (用于 Tauri)
- **uv** (Python 包管理器)

## 快速开始

### 1. 克隆项目

```bash
git clone <repository-url>
cd redata-app
```

### 2. 安装前端依赖

```bash
npm install
```

### 3. 安装后端依赖

```bash
cd backend
uv sync
cd ..
```

### 4. 启动开发服务器

#### 方式 1：分别启动（推荐用于开发）

```bash
# 终端 1：启动后端服务器
cd backend
uv run python run.py

# 终端 2：启动前端开发服务器
npm run dev
```

#### 方式 2：启动 Tauri 开发模式

```bash
npm run tauri:dev
```

### 5. 访问应用

- **前端**: http://localhost:3000
- **后端 API**: http://127.0.0.1:8000
- **API 文档**: http://127.0.0.1:8000/docs

## 可用命令

### 前端命令

```bash
npm run dev          # 启动 Nuxt 开发服务器
npm run build        # 构建生产版本
npm run generate     # 生成静态文件
npm run preview      # 预览生产构建
npm run tauri:dev    # 启动 Tauri 开发模式
npm run tauri:build  # 构建 Tauri 应用
```

### 后端命令

```bash
cd backend
uv sync              # 安装/更新依赖
uv run python run.py # 启动开发服务器
```

## API 文档

FastAPI 自动生成交互式 API 文档：

- **Swagger UI**: http://127.0.0.1:8000/docs
- **ReDoc**: http://127.0.0.1:8000/redoc

## 数据库

数据库文件位置：`backend/data/app.db`

首次运行时会自动创建数据库和表结构。

## 开发进度

- [x] Phase 1: 项目初始化（Nuxt 4 + Tauri 2）
- [x] Phase 2: 数据库和基础服务（Python FastAPI）
- [x] Phase 3: AI 集成和 Excel 解析（两阶段处理方案）
- [x] Phase 4: 前端基础架构（布局、API 客户端、状态管理）
- [x] Phase 5: 前端 - 项目管理（项目列表、创建、切换）
- [x] Phase 6: 前端 - 字段定义（AI 辅助字段生成工作流）
- [x] Phase 7: 前端 - 处理界面（文件处理、进度显示）
- [x] Phase 8: 前端 - 结果页面（数据展示、编辑、导出）
- [x] Phase 9: UI 优化（全局标签页、卡片布局、固定表头）
- [x] Phase 10: 后端 API 集成（AI 字段生成接口）

**总进度**: 10/10 (100%)

## 文档

完整的项目文档位于 `prd/` 目录：

- `prd.md` - 产品需求和业务逻辑
- `design.md` - UI/UX 设计及 ASCII 图表
- `plan.md` - 实施计划
- `dev.md` - 技术细节和架构
- `README.md` - 文档索引

## 许可证

[待定]

## 贡献

欢迎贡献！请先阅读贡献指南。

## 联系方式

[待定]
