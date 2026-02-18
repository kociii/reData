// Rust 后端模块入口 - DDD 架构

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

use axum::Router;
use std::sync::Arc;

/// 应用状态
pub struct AppState {
    // 这里将存储依赖注入的服务
}

/// 创建后端应用
pub async fn create_app() -> Router {
    // 初始化基础设施层
    let db = infrastructure::persistence::database::init_database().await
        .expect("Failed to initialize database");

    // 创建仓储实现
    // let project_repo = Arc::new(infrastructure::persistence::repositories::ProjectRepositoryImpl::new(db.clone()));

    // 创建用例
    // let create_project_use_case = Arc::new(application::use_cases::project::CreateProjectUseCase::new(project_repo));

    // 创建路由
    presentation::api::create_routes()
}
