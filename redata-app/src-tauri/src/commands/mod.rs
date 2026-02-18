// Commands 模块 - Tauri Commands 入口
//
// 这个模块包含所有的 Tauri Commands，用于前端调用后端功能
// 使用 Tauri Commands 模式，零网络开销，直接函数调用

pub mod projects;

// 重新导出所有 commands，方便在 lib.rs 中注册
pub use projects::*;
