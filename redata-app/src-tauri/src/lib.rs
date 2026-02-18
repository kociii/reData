// Tauri 应用库

// 导出 Rust 后端模块
pub mod backend;
// 导出 Tauri Commands 模块
pub mod commands;

use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

// 全局变量存储后端进程（旧的 HTTP 后端，已弃用）
#[allow(dead_code)]
static BACKEND_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 启动 Rust 后端服务器（在独立线程中）
// 注意: 此函数已弃用，现在使用 Tauri Commands 而不是 HTTP API
#[allow(dead_code)]
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

// 停止后端服务器（已弃用）
#[allow(dead_code)]
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
    // 初始化 tokio runtime（用于异步数据库操作）
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    // 初始化数据库连接
    println!("🔌 正在连接数据库...");
    let db = runtime.block_on(async {
        backend::infrastructure::persistence::database::init_database()
            .await
            .expect("Failed to initialize database")
    });
    println!("✅ 数据库连接成功");

    // 运行数据库迁移
    println!("🔄 正在运行数据库迁移...");
    runtime.block_on(async {
        backend::infrastructure::persistence::migrations::run_migrations(&db)
            .await
            .expect("Failed to run migrations")
    });
    println!("✅ 数据库迁移完成");

    // 将数据库连接包装为 Arc，用于在多个 commands 之间共享
    let db = Arc::new(db);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            greet,
            // 项目管理 Commands
            commands::get_projects,
            commands::create_project,
            commands::get_project,
            commands::update_project,
            commands::delete_project,
            // 字段管理 Commands
            commands::get_fields,
            commands::get_all_fields,
            commands::create_field,
            commands::update_field,
            commands::delete_field,
            commands::restore_field,
            commands::generate_field_metadata,
            // AI 配置 Commands
            commands::get_ai_configs,
            commands::get_ai_config,
            commands::get_default_ai_config,
            commands::create_ai_config,
            commands::update_ai_config,
            commands::delete_ai_config,
            commands::set_default_ai_config,
            commands::test_ai_connection,
            // AI 服务 Commands
            commands::analyze_column_mapping,
            commands::ai_generate_field_metadata,
            // 记录管理 Commands
            commands::insert_record,
            commands::insert_records_batch,
            commands::query_records,
            commands::get_record,
            commands::update_record,
            commands::delete_record,
            commands::delete_project_records,
            commands::get_record_count,
            commands::check_duplicate,
            // Excel 解析 Commands
            commands::get_excel_sheets,
            commands::preview_excel,
            // 任务管理 Commands
            commands::create_processing_task,
            commands::get_processing_task,
            commands::list_processing_tasks,
            commands::update_task_status,
            commands::create_batch,
            commands::get_batches,
            // 处理 Commands
            commands::start_processing,
            commands::pause_processing_task,
            commands::resume_processing_task,
            commands::cancel_processing_task,
            // 统计 Commands
            commands::get_project_statistics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
