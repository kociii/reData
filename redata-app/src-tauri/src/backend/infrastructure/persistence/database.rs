// 数据库连接模块

use sea_orm::{Database, DatabaseConnection, DbErr};
use std::path::PathBuf;

/// 数据库配置
pub struct DatabaseConfig {
    pub database_url: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        // 默认数据库路径
        let db_path = get_default_db_path();
        Self {
            database_url: format!("sqlite://{}?mode=rwc", db_path.display()),
        }
    }
}

/// 获取默认数据库路径
fn get_default_db_path() -> PathBuf {
    // 在开发环境中，使用项目根目录下的 data 目录
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // 尝试找到 backend/data 目录
    if path.join("backend").join("data").exists() {
        path = path.join("backend").join("data");
    } else if path.parent().and_then(|p| p.parent()).map(|p| p.join("backend").join("data").exists()).unwrap_or(false) {
        // 如果在 src-tauri 目录中，向上两级找到 backend/data
        path = path.parent().unwrap().parent().unwrap().join("backend").join("data");
    } else {
        // 创建 data 目录
        path = path.join("data");
        std::fs::create_dir_all(&path).ok();
    }

    path.join("app.db")
}

/// 初始化数据库连接
pub async fn init_database() -> Result<DatabaseConnection, DbErr> {
    let config = DatabaseConfig::default();
    init_database_with_config(&config).await
}

/// 使用自定义配置初始化数据库
pub async fn init_database_with_config(config: &DatabaseConfig) -> Result<DatabaseConnection, DbErr> {
    tracing::info!("Connecting to database: {}", config.database_url);

    let db = Database::connect(&config.database_url).await?;

    tracing::info!("Database connected successfully");

    Ok(db)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert!(config.database_url.starts_with("sqlite://"));
    }
}
