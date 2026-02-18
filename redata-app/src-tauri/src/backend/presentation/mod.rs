// Presentation Layer - 表现层模块入口

pub mod api;
pub mod middleware;

use axum::Router;

/// 创建所有 API 路由
pub fn create_routes() -> Router {
    Router::new()
        .nest("/api", api::create_api_routes())
        .route("/health", axum::routing::get(health_check))
}

/// 健康检查端点
async fn health_check() -> &'static str {
    "OK"
}
