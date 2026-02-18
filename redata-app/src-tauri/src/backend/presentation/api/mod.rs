// API routes module

pub mod ai_configs;
pub mod fields;
pub mod files;
pub mod processing;
pub mod projects;
pub mod results;

use axum::Router;

// 重新导出 AppState
pub use projects::AppState;

/// 创建所有 API 路由
pub fn create_api_routes() -> Router<AppState> {
    Router::new()
        .nest("/projects", projects::create_routes())
        // TODO: 实现其他路由
        // .nest("/fields", fields::create_routes())
        // .nest("/ai-configs", ai_configs::create_routes())
        // .nest("/files", files::create_routes())
        // .nest("/processing", processing::create_routes())
        // .nest("/results", results::create_routes())
}
