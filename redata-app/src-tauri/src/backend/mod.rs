// Rust 后端模块入口 - DDD 架构

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

use axum::Router;
use infrastructure::config::init_logging;
use std::net::SocketAddr;

/// 应用状态
pub struct AppState {
    // 这里将存储依赖注入的服务
}

/// 创建后端应用
pub async fn create_app() -> Router {
    // 初始化日志
    init_logging();

    tracing::info!("Creating backend application");

    // TODO: 初始化数据库
    // let db = infrastructure::persistence::database::init_database().await
    //     .expect("Failed to initialize database");

    // TODO: 创建仓储实现
    // let project_repo = Arc::new(infrastructure::persistence::repositories::ProjectRepositoryImpl::new(db.clone()));

    // TODO: 创建用例
    // let create_project_use_case = Arc::new(application::use_cases::project::CreateProjectUseCase::new(project_repo));

    // 创建路由
    presentation::create_routes()
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
