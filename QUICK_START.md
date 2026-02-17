# 快速启动指南

## 目录结构

```
reData/                    # Git 仓库根目录
├── prd/                   # 项目文档
├── CLAUDE.md             # 开发指南
└── redata-app/           # 应用代码目录 ⭐
    ├── backend/          # Python 后端
    ├── pages/            # Nuxt 前端
    ├── src-tauri/        # Tauri 配置
    └── package.json      # Node 依赖
```

## 启动步骤

### 1. 进入应用目录

```bash
cd redata-app
```

### 2. 首次运行（安装依赖）

```bash
# 安装前端依赖
npm install

# 安装后端依赖
cd backend
uv sync
cd ..
```

### 3. 启动应用

#### 方式 1：Tauri 开发模式（推荐）

```bash
npm run tauri:dev
```

这将：
- ✅ 自动启动 Python FastAPI 后端
- ✅ 启动 Nuxt 前端开发服务器
- ✅ 打开 Tauri 桌面窗口

#### 方式 2：分别启动（用于调试）

```bash
# 终端 1：启动后端
cd backend
uv run python run.py

# 终端 2：启动前端
npm run dev
```

## 常见问题

### Q: 提示 "Missing script: tauri:dev"

**原因**：您在错误的目录（reData 而不是 redata-app）

**解决**：
```bash
cd redata-app
npm run tauri:dev
```

### Q: 后端启动失败

**检查 Python**：
```bash
python3 --version  # 需要 3.11+
```

**安装后端依赖**：
```bash
cd backend
uv sync
```

### Q: 端口被占用

**清理端口 8000**：
```bash
lsof -ti:8000 | xargs kill -9
```

## 访问地址

- **前端开发服务器**: http://localhost:3000
- **后端 API**: http://127.0.0.1:8000
- **API 文档**: http://127.0.0.1:8000/docs

## 构建生产版本

```bash
cd redata-app
npm run tauri:build
```

构建产物位置：
- **macOS**: `src-tauri/target/release/bundle/macos/reData.app`
- **Windows**: `src-tauri/target/release/bundle/msi/reData.msi`
- **Linux**: `src-tauri/target/release/bundle/appimage/reData.AppImage`

## 开发工作流

```bash
# 1. 进入应用目录
cd redata-app

# 2. 启动开发模式
npm run tauri:dev

# 3. 修改代码（自动热重载）
# - 前端代码：pages/、components/、stores/
# - 后端代码：backend/src/redata/

# 4. 查看 API 文档
open http://127.0.0.1:8000/docs

# 5. 提交代码
cd ..  # 回到 reData 目录
git add .
git commit -m "your message"
```

## 目录说明

| 目录 | 说明 | 用途 |
|------|------|------|
| `reData/` | Git 仓库根目录 | 文档、提交历史 |
| `redata-app/` | 应用代码目录 | 所有开发工作在这里 |
| `redata-app/backend/` | Python 后端 | FastAPI、数据库、AI 集成 |
| `redata-app/pages/` | Nuxt 页面 | 前端页面组件 |
| `redata-app/stores/` | Pinia stores | 状态管理 |
| `redata-app/src-tauri/` | Tauri 配置 | 桌面应用配置 |

## 下一步

- 阅读 `redata-app/README.md` 了解详细信息
- 查看 `CLAUDE.md` 了解开发指南
- 查看 `prd/` 目录了解产品需求
