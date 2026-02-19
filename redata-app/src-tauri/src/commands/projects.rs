// 项目管理 Tauri Commands
//
// 这个模块实现了项目管理的所有 Tauri Commands
// 前端通过 invoke() 调用这些命令，零网络开销

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::backend::infrastructure::{
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
#[derive(Debug, Serialize, Clone)]
pub struct ProjectResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub group_id: Option<i32>,
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
            group_id: model.group_id,
            dedup_enabled: model.dedup_enabled,
            dedup_fields,
            dedup_strategy: model.dedup_strategy,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

/// 获取项目列表
#[tauri::command]
pub async fn get_projects(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
) -> Result<Vec<ProjectResponse>, String> {
    let projects = Project::find()
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let responses: Vec<ProjectResponse> = projects.into_iter().map(Into::into).collect();

    Ok(responses)
}

/// 创建项目
#[tauri::command]
pub async fn create_project(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    name: String,
    description: Option<String>,
) -> Result<ProjectResponse, String> {
    // 检查项目名称是否已存在
    let existing = Project::find()
        .filter(project::Column::Name.eq(&name))
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if existing.is_some() {
        return Err(format!("Project with name '{}' already exists", name));
    }

    // 创建项目
    let now = chrono::Utc::now();
    let project = project::ActiveModel {
        name: Set(name),
        description: Set(description),
        dedup_enabled: Set(true),
        dedup_fields: Set(None),
        dedup_strategy: Set("skip".to_string()),
        created_at: Set(now),
        updated_at: Set(None),
        ..Default::default()
    };

    let result = project
        .insert(db.inner().as_ref())
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(result.into())
}

/// 获取单个项目
#[tauri::command]
pub async fn get_project(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
) -> Result<ProjectResponse, String> {
    let project = Project::find_by_id(id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Project {} not found", id))?;

    Ok(project.into())
}

/// 更新项目
#[tauri::command]
pub async fn update_project(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
    name: Option<String>,
    description: Option<String>,
    dedup_enabled: Option<bool>,
    dedup_fields: Option<Vec<String>>,
    dedup_strategy: Option<String>,
) -> Result<ProjectResponse, String> {
    // 查找项目
    let project = Project::find_by_id(id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Project {} not found", id))?;

    // 更新项目
    let mut active: project::ActiveModel = project.into();

    if let Some(name) = name {
        active.name = Set(name);
    }
    if let Some(description) = description {
        active.description = Set(Some(description));
    }
    if let Some(dedup_enabled) = dedup_enabled {
        active.dedup_enabled = Set(dedup_enabled);
    }
    if let Some(dedup_fields) = dedup_fields {
        let json_str = if dedup_fields.is_empty() {
            None
        } else {
            serde_json::to_string(&dedup_fields).ok()
        };
        active.dedup_fields = Set(json_str);
    }
    if let Some(dedup_strategy) = dedup_strategy {
        active.dedup_strategy = Set(dedup_strategy);
    }

    active.updated_at = Set(Some(chrono::Utc::now()));

    let result = active
        .update(db.inner().as_ref())
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(result.into())
}

/// 删除项目
#[tauri::command]
pub async fn delete_project(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
) -> Result<(), String> {
    // 查找项目
    let project = Project::find_by_id(id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Project {} not found", id))?;

    // 删除项目
    let active: project::ActiveModel = project.into();
    active
        .delete(db.inner().as_ref())
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(())
}
