# 自动启动功能说明

## 功能概述

reData 现在是一个真正的桌面客户端应用，双击图标即可自动启动，无需手动启动后端服务器。

## 工作原理

1. **Tauri 启动时**：
   - 自动检测并启动 Python FastAPI 后端服务器
   - 等待 2 秒确保后端服务器完全启动
   - 启动前端 Nuxt 应用

2. **应用运行时**：
   - 前端通过 HTTP API 与后端通信
   - 后端服务器在后台运行（http://127.0.0.1:8000）

3. **应用关闭时**：
   - 自动停止后端服务器进程
   - 清理所有资源

## 实现细节

### Rust 代码（src-tauri/src/lib.rs）

```rust
// 启动 FastAPI 后端服务器
fn start_backend_server() -> Result<Child, std::io::Error> {
    let python_cmd = if cfg!(target_os = "windows") {
        "python"
    } else {
        "python3"
    };

    let backend_dir = std::env::current_dir()
        .unwrap()
        .join("backend");

    let child = Command::new(python_cmd)
        .arg("run.py")
        .current_dir(backend_dir)
        .spawn()?;

    Ok(child)
}
```

### 生命周期管理

- **启动**：`run()` 函数中调用 `start_backend_server()`
- **停止**：窗口关闭事件中调用 `stop_backend_server()`

## 使用方法

### 开发模式

```bash
npm run tauri:dev
```

这将：
1. 自动启动 FastAPI 后端服务器
2. 启动 Nuxt 开发服务器
3. 打开 Tauri 窗口

### 生产模式

```bash
npm run tauri:build
```

构建后的应用：
- **macOS**: `src-tauri/target/release/bundle/macos/reData.app`
- **Windows**: `src-tauri/target/release/bundle/msi/reData.msi`
- **Linux**: `src-tauri/target/release/bundle/appimage/reData.AppImage`

双击应用图标即可启动，无需任何额外操作。

## 前提条件

### 开发环境

- Python 3.11+ 已安装
- uv 包管理器已安装
- 后端依赖已安装（`cd backend && uv sync`）

### 生产环境

打包时需要确保：
1. Python 运行时包含在应用包中，或
2. 用户系统已安装 Python 3.11+

## 配置选项

### 后端端口

默认端口：8000

如需修改，需要同时更新：
1. `backend/run.py` 中的端口配置
2. 前端 API 调用的基础 URL

### 启动延迟

默认等待 2 秒确保后端启动完成。

如需调整，修改 `src-tauri/src/lib.rs`：

```rust
std::thread::sleep(std::time::Duration::from_secs(2));
```

## 故障排查

### 后端启动失败

**症状**：应用启动但无法连接到后端

**可能原因**：
1. Python 未安装或不在 PATH 中
2. 后端依赖未安装
3. 端口 8000 被占用

**解决方法**：
```bash
# 检查 Python
python3 --version

# 安装后端依赖
cd backend
uv sync

# 检查端口占用
lsof -i:8000  # macOS/Linux
netstat -ano | findstr :8000  # Windows
```

### 应用关闭后后端仍在运行

**症状**：关闭应用后端口仍被占用

**解决方法**：
```bash
# 手动停止进程
lsof -ti:8000 | xargs kill -9  # macOS/Linux
taskkill /F /PID <PID>  # Windows
```

## 未来改进

1. **Python 运行时打包**：
   - 使用 PyInstaller 将 Python 后端打包为独立可执行文件
   - 无需用户安装 Python

2. **健康检查**：
   - 启动后检查后端健康状态
   - 显示启动进度

3. **错误处理**：
   - 更好的错误提示
   - 自动重试机制

4. **日志记录**：
   - 记录后端启动日志
   - 便于故障排查
