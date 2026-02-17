// Tauri 应用库
// 后端服务使用 Python + FastAPI 实现

use std::process::{Command, Child};
use std::sync::Mutex;

// 全局变量存储后端进程
static BACKEND_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 启动 FastAPI 后端服务器
fn start_backend_server() -> Result<Child, std::io::Error> {
    #[cfg(target_os = "windows")]
    let python_cmd = "python";

    #[cfg(not(target_os = "windows"))]
    let python_cmd = "python3";

    // 获取后端目录路径
    let backend_dir = std::env::current_dir()
        .unwrap()
        .join("backend");

    // 启动 FastAPI 服务器
    let child = Command::new(python_cmd)
        .arg("run.py")
        .current_dir(backend_dir)
        .spawn()?;

    println!("FastAPI 后端服务器已启动，PID: {}", child.id());
    Ok(child)
}

// 停止后端服务器
fn stop_backend_server() {
    if let Ok(mut process) = BACKEND_PROCESS.lock() {
        if let Some(mut child) = process.take() {
            let _ = child.kill();
            println!("FastAPI 后端服务器已停止");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 启动后端服务器
    match start_backend_server() {
        Ok(child) => {
            if let Ok(mut process) = BACKEND_PROCESS.lock() {
                *process = Some(child);
            }
        }
        Err(e) => {
            eprintln!("启动后端服务器失败: {}", e);
        }
    }

    // 等待后端服务器启动
    std::thread::sleep(std::time::Duration::from_secs(2));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
