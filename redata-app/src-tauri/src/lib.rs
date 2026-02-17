// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod db;
mod models;
mod services;
mod commands;

use db::init_db;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化数据库
    init_db().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
