// Commands 模块 - Tauri Commands 入口
//
// 这个模块包含所有的 Tauri Commands，用于前端调用后端功能
// 使用 Tauri Commands 模式，零网络开销，直接函数调用

pub mod projects;
pub mod fields;
pub mod ai_configs;
pub mod ai_service;
pub mod ai_utils;
pub mod records;
pub mod excel;
pub mod tasks;
pub mod processing;

// 重新导出所有 commands，方便在 lib.rs 中注册
pub use projects::*;
pub use fields::*;
pub use ai_configs::*;
pub use ai_service::*;
pub use records::*;
pub use excel::*;
pub use tasks::*;
pub use processing::*;
