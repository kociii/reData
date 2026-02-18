// persistence module

pub mod database;
pub mod models;
pub mod repositories;

// 导出常用类型
pub use database::{init_database, init_database_with_config, DatabaseConfig};
