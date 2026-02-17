# 架构变更说明

## 变更时间
2026-02-17

## 变更原因

考虑到数据处理和大模型调用的需求，将后端从 Rust 改为 Python 实现。

**Python 的优势**：
1. **数据处理生态丰富** - pandas、numpy、openpyxl 等成熟库
2. **AI/ML 生态完善** - OpenAI SDK、LangChain 等
3. **开发效率高** - 快速迭代，易于调试
4. **Excel 处理成熟** - openpyxl、pandas 对 Excel 支持完善

## 新架构

### 技术栈

**前端**：
- Nuxt 3.18+ + Vue 3 + TypeScript
- Nuxt UI 3.x
- Pinia 状态管理
- Tauri 2.x（桌面应用框架）

**后端**：
- FastAPI（Python Web 框架）
- SQLAlchemy（ORM）
- OpenAI SDK（AI 集成）
- pandas + openpyxl（Excel 处理）
- uvicorn（ASGI 服务器）

**数据库**：
- SQLite 3.40+

**包管理**：
- uv（Python 包管理器）

### 通信方式

```
┌─────────────────────────────────────┐
│         Tauri Desktop App           │
│  ┌───────────────────────────────┐  │
│  │      Nuxt 3 Frontend          │  │
│  │   (http://localhost:3000)     │  │
│  └───────────┬───────────────────┘  │
│              │ HTTP API              │
│              ▼                       │
│  ┌───────────────────────────────┐  │
│  │   FastAPI Backend Server      │  │
│  │   (http://127.0.0.1:8000)     │  │
│  └───────────┬───────────────────┘  │
│              │                       │
│              ▼                       │
│  ┌───────────────────────────────┐  │
│  │   SQLite Database             │  │
│  │   (data/app.db)               │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

### 项目结构

```
redata-app/
├── frontend/                    # Nuxt 3 前端（原根目录）
│   ├── app/
│   ├── pages/
│   ├── stores/
│   ├── components/
│   └── nuxt.config.ts
├── backend/                     # Python 后端（新增）
│   ├── src/
│   │   └── redata/
│   │       ├── api/            # API 路由
│   │       │   ├── projects.py
│   │       │   ├── fields.py
│   │       │   └── ai_configs.py
│   │       ├── db/             # 数据库配置
│   │       │   └── base.py
│   │       ├── models/         # 数据模型
│   │       │   ├── project.py
│   │       │   └── schemas.py
│   │       ├── services/       # 业务服务
│   │       └── main.py         # FastAPI 应用
│   ├── pyproject.toml          # Python 依赖
│   └── run.py                  # 启动脚本
└── src-tauri/                   # Tauri 配置（简化）
    ├── tauri.conf.json
    └── src/
        ├── main.rs             # 启动 FastAPI 服务器
        └── lib.rs
```

## 已实现功能

### 数据库模型（SQLAlchemy）

- ✅ `Project` - 项目模型
- ✅ `ProjectField` - 字段定义模型
- ✅ `ProcessingTask` - 任务模型
- ✅ `AiConfig` - AI 配置模型
- ✅ `Batch` - 批次模型

### Pydantic Schemas

- ✅ ProjectCreate/Update/Response
- ✅ ProjectFieldCreate/Update/Response
- ✅ ProcessingTaskCreate/Response
- ✅ AiConfigCreate/Update/Response

### API 路由

**项目管理** (`/api/projects`):
- ✅ POST `/` - 创建项目
- ✅ GET `/` - 列出所有项目
- ✅ GET `/{project_id}` - 获取单个项目
- ✅ PUT `/{project_id}` - 更新项目
- ✅ DELETE `/{project_id}` - 删除项目

**字段管理** (`/api/fields`):
- ✅ POST `/` - 创建字段
- ✅ GET `/project/{project_id}` - 获取项目字段
- ✅ PUT `/{field_id}` - 更新字段
- ✅ DELETE `/{field_id}` - 删除字段

**AI 配置** (`/api/ai-configs`):
- ✅ POST `/` - 创建配置
- ✅ GET `/` - 列出所有配置
- ✅ GET `/{config_id}` - 获取单个配置
- ✅ PUT `/{config_id}` - 更新配置
- ✅ DELETE `/{config_id}` - 删除配置

### 核心功能

- ✅ 数据库自动初始化
- ✅ CORS 配置（支持 Tauri 前端）
- ✅ RESTful API 设计
- ✅ 请求验证（Pydantic）
- ✅ 错误处理
- ✅ API 文档（自动生成）

## 开发命令

### 后端开发

```bash
cd backend

# 安装依赖
uv sync

# 启动开发服务器
uv run python run.py

# 访问 API 文档
open http://127.0.0.1:8000/docs
```

### 前端开发

```bash
# 启动 Nuxt 开发服务器
npm run dev

# 启动 Tauri 开发模式
npm run tauri:dev
```

## API 文档

FastAPI 自动生成交互式 API 文档：
- Swagger UI: http://127.0.0.1:8000/docs
- ReDoc: http://127.0.0.1:8000/redoc

## 验收标准

✅ **后端服务**：
- FastAPI 服务器成功启动在 http://127.0.0.1:8000
- API 根路径返回正确的响应
- 数据库自动初始化
- 所有 API 路由正常工作

✅ **数据模型**：
- SQLAlchemy 模型定义完整
- Pydantic schemas 验证正确
- 数据库表自动创建

✅ **API 功能**：
- 项目 CRUD 操作完整
- 字段 CRUD 操作完整
- AI 配置 CRUD 操作完整

## 下一步

### Phase 3：AI 集成和 Excel 解析（继续）

**需要实现**：
1. AI 客户端服务（services/ai_client.py）
   - OpenAI SDK 集成
   - Prompt 动态生成
   - 错误重试机制

2. Excel 解析服务（services/excel_parser.py）
   - 使用 openpyxl 读取 Excel
   - 遍历 sheets 和行
   - 表头识别

3. 数据提取服务（services/extractor.py）
   - 协调 AI 和 Excel 解析
   - 数据提取流程
   - 进度跟踪

4. API 路由扩展
   - 文件上传 API
   - 处理任务 API
   - 结果查询 API

## 技术优势

1. **开发效率** - Python 快速迭代，FastAPI 自动生成文档
2. **生态丰富** - pandas、OpenAI SDK 等成熟库
3. **易于调试** - Python 调试工具完善
4. **类型安全** - Pydantic 提供运行时类型验证
5. **性能优化** - 可以使用 asyncio 进行异步处理

## 注意事项

1. **Python 环境** - 需要 Python 3.11+
2. **依赖管理** - 使用 uv 管理依赖
3. **数据库** - SQLite 文件位于 `data/app.db`
4. **API 端口** - 后端默认端口 8000，前端 3000
