// 项目 API 路由
// 注意: 此模块已弃用，现在使用 Tauri Commands 而不是 HTTP API

#[allow(unused_imports)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::backend::infrastructure::{
    config::{AppError, Result},
    persistence::models::{project, Project},
};

/// 项目创建请求
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

/// 项目更新请求
#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub dedup_enabled: Option<bool>,
    pub dedup_fields: Option<Vec<String>>,
    pub dedup_strategy: Option<String>,
}

/// 项目响应
#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub dedup_enabled: bool,
    pub dedup_fields: Vec<String>,
    pub dedup_strategy: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl From<project::Model> for ProjectResponse {
    fn from(model: project::Model) -> Self {
        let dedup_fields = model.get_dedup_fields();
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            dedup_enabled: model.dedup_enabled,
            dedup_fields,
            dedup_strategy: model.dedup_strategy,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_projects).post(create_project))
        .route("/:id", get(get_project).put(update_project).delete(delete_project))
}

/// 获取项目列表
async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProjectResponse>>> {
    let projects = Project::find()
        .all(state.db.as_ref())
        .await
        .map_err(AppError::from)?;

    let responses: Vec<ProjectResponse> = projects.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// 创建项目
async fn create_project(
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>)> {
    // 检查项目名称是否已存在
    let existing = Project::find()
        .filter(project::Column::Name.eq(&req.name))
        .one(state.db.as_ref())
        .await
        .map_err(AppError::from)?;

    if existing.is_some() {
        return Err(AppError::AlreadyExists(format!(
            "Project with name '{}' already exists",
            req.name
        )));
    }

    // 创建项目
    let now = chrono::Utc::now();
    let project = project::ActiveModel {
        name: Set(req.name),
        description: Set(req.description),
        dedup_enabled: Set(true),
        dedup_fields: Set(None),
        dedup_strategy: Set("skip".to_string()),
        created_at: Set(now),
        updated_at: Set(None),
        ..Default::default()
    };

    let result = project.insert(state.db.as_ref()).await.map_err(AppError::from)?;

    Ok((StatusCode::CREATED, Json(result.into())))
}

/// 获取单个项目
async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ProjectResponse>> {
    let project = Project::find_by_id(id)
        .one(state.db.as_ref())
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Project {} not found", id)))?;

    Ok(Json(project.into()))
}

/// 更新项目
async fn update_project(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>> {
    // 查找项目
    let project = Project::find_by_id(id)
        .one(state.db.as_ref())
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Project {} not found", id)))?;

    // 更新项目
    let mut active: project::ActiveModel = project.into();

    if let Some(name) = req.name {
        active.name = Set(name);
    }
    if let Some(description) = req.description {
        active.description = Set(Some(description));
    }
    if let Some(dedup_enabled) = req.dedup_enabled {
        active.dedup_enabled = Set(dedup_enabled);
    }
    if let Some(dedup_fields) = req.dedup_fields {
        let json_str = if dedup_fields.is_empty() {
            None
        } else {
            serde_json::to_string(&dedup_fields).ok()
        };
        active.dedup_fields = Set(json_str);
    }
    if let Some(dedup_strategy) = req.dedup_strategy {
        active.dedup_strategy = Set(dedup_strategy);
    }

    active.updated_at = Set(Some(chrono::Utc::now()));

    let result = active.update(state.db.as_ref()).await.map_err(AppError::from)?;

    Ok(Json(result.into()))
}

/// 删除项目
async fn delete_project(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode> {
    // 查找项目
    let project = Project::find_by_id(id)
        .one(state.db.as_ref())
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Project {} not found", id)))?;

    // 删除项目
    let active: project::ActiveModel = project.into();
    active.delete(state.db.as_ref()).await.map_err(AppError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
