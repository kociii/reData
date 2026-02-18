// Tauri 应用库

// 导出 Rust 后端模块
pub mod backend;

use std::process::{Child, Command};
use std::sync::Mutex;

// 全局变量存储后端进程
static BACKEND_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 启动 Rust 后端服务器（在独立线程中）
fn start_rust_backend() {
    std::thread::spawn(|| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            println!("正在启动 Rust 后端服务器...");
            if let Err(e) = backend::run_server(8001).await {
                eprintln!("Rust 后端服务器错误: {}", e);
            }
        });
    });
}

// 获取后端目录路径（用于 Python 后端）
fn get_backend_dir() -> std::path::PathBuf {
    // 尝试多种可能的路径
    let possible_paths = vec![
        // 开发模式: redata-app/src-tauri -> redata-app/backend
        std::env::current_dir()
            .unwrap()
            .parent()
            .map(|p| p.join("backend")),
        // 从可执行文件目录查找
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .and_then(|p| p.parent().map(|p| p.join("backend"))),
        // 当前目录的 backend 子目录
        Some(std::env::current_dir().unwrap().join("backend")),
    ];

    for path_opt in possible_paths {
        if let Some(path) = path_opt {
            if path.exists() && path.join("run.py").exists() {
                return path;
            }
        }
    }

    // 默认返回当前目录的 backend 子目录
    std::env::current_dir().unwrap().join("backend")
}

// 启动 Python FastAPI 后端服务器（备用）
#[allow(dead_code)]
fn start_python_backend_server() -> Result<Child, std::io::Error> {
    let backend_dir = get_backend_dir();

    // 检查后端目录是否存在
    if !backend_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("后端目录不存在: {:?}", backend_dir),
        ));
    }

    let run_py = backend_dir.join("run.py");
    if !run_py.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("run.py 不存在: {:?}", run_py),
        ));
    }

    println!("后端目录: {:?}", backend_dir);

    // 优先使用 uv 运行（如果可用）
    let uv_path = backend_dir.join(".venv");
    let use_uv = uv_path.exists();

    let child = if use_uv {
        // 使用 uv run 启动
        Command::new("uv")
            .arg("run")
            .arg("python")
            .arg("run.py")
            .current_dir(&backend_dir)
            .spawn()?
    } else {
        // 使用系统 Python
        #[cfg(target_os = "windows")]
        let python_cmd = "python";

        #[cfg(not(target_os = "windows"))]
        let python_cmd = "python3";

        Command::new(python_cmd)
            .arg("run.py")
            .current_dir(&backend_dir)
            .spawn()?
    };

    println!("Python FastAPI 后端服务器已启动，PID: {}", child.id());
    Ok(child)
}

// 停止后端服务器
fn stop_backend_server() {
    if let Ok(mut process) = BACKEND_PROCESS.lock() {
        if let Some(mut child) = process.take() {
            let _ = child.kill();
            println!("后端服务器已停止");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 启动 Rust 后端服务器（推荐）
    println!("🚀 启动 Rust 后端服务器（端口 8001）...");
    start_rust_backend();

    // 可选：同时启动 Python 后端（如果需要）
    // match start_python_backend_server() {
    //     Ok(child) => {
    //         if let Ok(mut process) = BACKEND_PROCESS.lock() {
    //             *process = Some(child);
    //         }
    //     }
    //     Err(e) => {
    //         eprintln!("启动 Python 后端服务器失败: {}", e);
    //     }
    // }

    // 等待后端服务器启动
    std::thread::sleep(std::time::Duration::from_secs(2));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet])
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                // 窗口关闭时停止后端服务器
                stop_backend_server();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
