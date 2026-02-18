// 任务管理 Tauri Commands
//
// 处理任务和批次的 CRUD 操作

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::Serialize;
use std::sync::Arc;

use crate::backend::infrastructure::persistence::models::{
    task, ProcessingTask, batch, Batch,
};

// ============ 响应结构 ============

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub task_id: String,
    pub project_id: i32,
    pub status: String,
    pub total_files: i32,
    pub processed_files: i32,
    pub total_rows: i32,
    pub processed_rows: i32,
    pub success_count: i32,
    pub error_count: i32,
    pub batch_number: Option<String>,
    pub source_files: Option<Vec<String>>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl From<task::Model> for TaskResponse {
    fn from(m: task::Model) -> Self {
        // 解析 source_files JSON 字符串为 Vec<String>
        let source_files = m.source_files.and_then(|s| {
            serde_json::from_str(&s).ok()
        });

        Self {
            task_id: m.id.clone(),
            project_id: m.project_id,
            status: m.status,
            total_files: m.total_files,
            processed_files: m.processed_files,
            total_rows: m.total_rows,
            processed_rows: m.processed_rows,
            success_count: m.success_count,
            error_count: m.error_count,
            batch_number: m.batch_number,
            source_files,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.map(|t| t.to_rfc3339()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BatchResponse {
    pub id: i32,
    pub batch_number: String,
    pub project_id: i32,
    pub file_count: i32,
    pub record_count: i32,
    pub created_at: String,
}

impl From<batch::Model> for BatchResponse {
    fn from(m: batch::Model) -> Self {
        Self {
            id: m.id,
            batch_number: m.batch_number,
            project_id: m.project_id,
            file_count: m.file_count,
            record_count: m.record_count,
            created_at: m.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ListTasksResponse {
    pub tasks: Vec<TaskResponse>,
    pub total: u64,
}

// ============ Tauri Commands ============

/// 创建处理任务（生成 UUID + batch_number）
#[tauri::command]
pub async fn create_processing_task(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    total_files: i32,
) -> Result<TaskResponse, String> {
    let task_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    // 生成 batch_number: BATCH_YYYYMMDD_NNN
    let date_str = now.format("%Y%m%d").to_string();
    let count = ProcessingTask::find()
        .filter(task::Column::ProjectId.eq(project_id))
        .filter(task::Column::BatchNumber.starts_with(&format!("BATCH_{}", date_str)))
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .len();
    let batch_number = format!("BATCH_{}_{:03}", date_str, count + 1);

    let new_task = task::ActiveModel {
        id: Set(task_id),
        project_id: Set(project_id),
        status: Set("pending".to_string()),
        total_files: Set(total_files),
        processed_files: Set(0),
        total_rows: Set(0),
        processed_rows: Set(0),
        success_count: Set(0),
        error_count: Set(0),
        batch_number: Set(Some(batch_number)),
        source_files: Set(None),
        created_at: Set(now),
        updated_at: Set(None),
    };

    let result = new_task
        .insert(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.into())
}

/// 获取任务状态
#[tauri::command]
pub async fn get_processing_task(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    task_id: String,
) -> Result<TaskResponse, String> {
    let task = ProcessingTask::find_by_id(&task_id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("任务 {} 不存在", task_id))?;

    Ok(task.into())
}

/// 列出项目任务（支持 status 过滤）
#[tauri::command]
pub async fn list_processing_tasks(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    status: Option<String>,
) -> Result<ListTasksResponse, String> {
    let mut query = ProcessingTask::find()
        .filter(task::Column::ProjectId.eq(project_id));

    if let Some(st) = status {
        query = query.filter(task::Column::Status.eq(st));
    }

    let tasks = query
        .order_by_desc(task::Column::CreatedAt)
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    let total = tasks.len() as u64;
    let task_responses: Vec<TaskResponse> = tasks.into_iter().map(|t| t.into()).collect();

    Ok(ListTasksResponse {
        tasks: task_responses,
        total,
    })
}

/// 更新任务状态
#[tauri::command]
pub async fn update_task_status(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    task_id: String,
    status: String,
) -> Result<TaskResponse, String> {
    let task = ProcessingTask::find_by_id(&task_id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("任务 {} 不存在", task_id))?;

    let mut active: task::ActiveModel = task.into();
    active.status = Set(status);
    active.updated_at = Set(Some(chrono::Utc::now()));

    let result = active
        .update(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.into())
}

/// 创建批次
#[tauri::command]
pub async fn create_batch(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    batch_number: String,
    file_count: i32,
) -> Result<BatchResponse, String> {
    let new_batch = batch::ActiveModel {
        batch_number: Set(batch_number),
        project_id: Set(project_id),
        file_count: Set(file_count),
        record_count: Set(0),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    let result = new_batch
        .insert(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.into())
}

/// 获取项目批次列表
#[tauri::command]
pub async fn get_batches(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
) -> Result<Vec<BatchResponse>, String> {
    let batches = Batch::find()
        .filter(batch::Column::ProjectId.eq(project_id))
        .order_by_desc(batch::Column::CreatedAt)
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(batches.into_iter().map(|b| b.into()).collect())
}
