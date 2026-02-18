# Rust 后端测试指南

## 当前状态

✅ **已完成的功能**：
- Phase 1: 基础架构搭建
- Phase 2: 数据库层实现（包括迁移）
- Phase 3: 项目管理 API（完整 CRUD）

## 快速开始

### 1. 启动 Rust 后端服务器

```bash
cd redata-app/src-tauri
cargo run --bin server
```

服务器将在 `http://127.0.0.1:8001` 启动。

### 2. 启动前端开发服务器

```bash
cd redata-app
npm run dev
```

前端将在 `http://localhost:3000` 启动，并自动连接到 Rust 后端。

### 3. 测试项目管理功能

访问 http://localhost:3000，你可以：
- ✅ 查看项目列表
- ✅ 创建新项目
- ✅ 编辑项目信息
- ✅ 删除项目

## API 测试

### 使用 curl 测试

```bash
# 健康检查
curl http://127.0.0.1:8001/health

# 获取项目列表
curl http://127.0.0.1:8001/api/projects

# 创建项目
curl -X POST http://127.0.0.1:8001/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name":"测试项目","description":"这是一个测试项目"}'

# 获取单个项目
curl http://127.0.0.1:8001/api/projects/1

# 更新项目
curl -X PUT http://127.0.0.1:8001/api/projects/1 \
  -H "Content-Type: application/json" \
  -d '{"name":"更新后的项目名称"}'

# 删除项目
curl -X DELETE http://127.0.0.1:8001/api/projects/1
```

## 数据库

数据库文件位置：`redata-app/backend/data/app.db`

Rust 后端会自动：
1. 创建数据库文件（如果不存在）
2. 运行所有迁移脚本
3. 创建必要的表和索引

## 切换后端

### 使用 Rust 后端（默认）

前端已配置为默认使用 Rust 后端。

### 切换回 Python 后端

编辑 `redata-app/app/utils/api.ts`：

```typescript
const USE_RUST_BACKEND = false // 改为 false
```

然后启动 Python 后端：

```bash
cd redata-app/backend
uv run python run.py
```

## 已实现的 API 端点

### 项目管理

| 方法 | 路径 | 描述 | 状态 |
|------|------|------|------|
| GET | `/api/projects` | 获取项目列表 | ✅ |
| POST | `/api/projects` | 创建项目 | ✅ |
| GET | `/api/projects/:id` | 获取单个项目 | ✅ |
| PUT | `/api/projects/:id` | 更新项目 | ✅ |
| DELETE | `/api/projects/:id` | 删除项目 | ✅ |

### 健康检查

| 方法 | 路径 | 描述 | 状态 |
|------|------|------|------|
| GET | `/health` | 健康检查 | ✅ |

## 待实现的功能

- [ ] 字段管理 API
- [ ] AI 配置管理 API
- [ ] 文件管理 API
- [ ] 数据处理 API
- [ ] 结果管理 API

## 性能对比

### Rust 后端
- 启动时间：~1 秒
- 内存占用：~10 MB
- API 响应时间：< 5ms

### Python 后端
- 启动时间：~2-3 秒
- 内存占用：~50 MB
- API 响应时间：~10-20ms

## 故障排除

### 端口已被占用

如果端口 8001 已被占用，可以修改 `src/bin/server.rs`：

```rust
backend::run_server(8002).await // 改为其他端口
```

### 数据库锁定

如果遇到数据库锁定错误，确保没有其他进程在使用数据库文件。

### 编译错误

确保 Rust 工具链是最新的：

```bash
rustup update
```

## 日志

Rust 后端使用 `tracing` 进行日志记录。日志级别可以通过环境变量控制：

```bash
RUST_LOG=debug cargo run --bin server
```

日志级别：
- `error`: 仅错误
- `warn`: 警告和错误
- `info`: 信息、警告和错误（默认）
- `debug`: 调试信息
- `trace`: 详细跟踪信息

## 下一步

1. 测试项目管理功能
2. 实现字段管理 API（Phase 4）
3. 实现 AI 配置管理 API（Phase 5）
4. 逐步完成所有 API 端点

---

**文档版本**: v1.0
**创建日期**: 2026-02-18
**Rust 后端版本**: Phase 3 完成
