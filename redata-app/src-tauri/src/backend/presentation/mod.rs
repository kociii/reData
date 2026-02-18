// Presentation Layer - 表现层模块入口

pub mod api;
pub mod middleware;

use axum::Router;
use middleware::{create_cors_layer, create_trace_layer};

/// 创建所有 API 路由
pub fn create_routes() -> Router<api::AppState> {
    Router::new()
        .nest("/api", api::create_api_routes())
        .route("/health", axum::routing::get(health_check))
        .layer(create_trace_layer())
        .layer(create_cors_layer())
}

/// 健康检查端点
async fn health_check() -> &'static str {
    "OK"
}
