// Rust 后端模块入口 - DDD 架构

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

use axum::Router;
use infrastructure::{config::init_logging, persistence::init_database};
use presentation::api::AppState;
use std::{net::SocketAddr, sync::Arc};

/// 创建后端应用
pub async fn create_app() -> Router {
    // 初始化日志
    init_logging();

    tracing::info!("Creating backend application");

    // 初始化数据库
    let db = init_database().await.expect("Failed to initialize database");

    tracing::info!("Database initialized");

    // 创建应用状态
    let state = AppState { db: Arc::new(db) };

    // 创建路由
    presentation::create_routes().with_state(state)
}

/// 启动后端服务器
pub async fn run_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app().await;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
