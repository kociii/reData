// 任务管理 Tauri Commands
//
// 处理任务和批次的 CRUD 操作

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, Set, Statement,
};
use serde::Serialize;
use std::sync::Arc;

use crate::backend::infrastructure::persistence::models::{
    task, ProcessingTask,
    task_file_progress, TaskFileProgress, ProjectRecord,
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

// ============ 文件进度持久化 ============

#[derive(Debug, Serialize)]
pub struct FileProgressResponse {
    pub file_name: String,
    pub file_phase: String,
    pub sheets: Vec<SheetProgressResponse>,
    pub total_rows: i32,
    pub success_count: i32,
    pub error_count: i32,
}

#[derive(Debug, Serialize)]
pub struct SheetProgressResponse {
    pub sheet_name: String,
    pub sheet_phase: String,
    pub ai_confidence: Option<f32>,
    pub mapping_count: Option<i32>,
    pub success_count: i32,
    pub error_count: i32,
    pub total_rows: i32,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FullTaskProgressResponse {
    pub task_id: String,
    pub files: Vec<FileProgressResponse>,
}

/// 获取任务的完整进度（文件和 Sheet 级别）
#[tauri::command]
pub async fn get_task_full_progress(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    task_id: String,
) -> Result<FullTaskProgressResponse, String> {
    let progress_records = TaskFileProgress::find()
        .filter(task_file_progress::Column::TaskId.eq(&task_id))
        .order_by(task_file_progress::Column::Id, sea_orm::Order::Asc)
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    // 按文件分组
    let mut files_map: std::collections::HashMap<String, FileProgressResponse> =
        std::collections::HashMap::new();

    for record in progress_records {
        let file_name = record.file_name.clone();

        // 获取或创建文件记录
        let file_progress = files_map.entry(file_name.clone()).or_insert_with(|| {
            FileProgressResponse {
                file_name: file_name.clone(),
                file_phase: "waiting".to_string(),
                sheets: Vec::new(),
                total_rows: 0,
                success_count: 0,
                error_count: 0,
            }
        });

        if record.sheet_name.is_some() {
            // Sheet 级别记录 - 只收集到列表，不累加（避免与文件级别记录重复计数）
            file_progress.sheets.push(SheetProgressResponse {
                sheet_name: record.sheet_name.unwrap(),
                sheet_phase: record.sheet_phase.unwrap_or("waiting".to_string()),
                ai_confidence: record.ai_confidence,
                mapping_count: record.mapping_count,
                success_count: record.success_count,
                error_count: record.error_count,
                total_rows: record.total_rows,
                error_message: record.error_message,
            });
        } else {
            // 文件级别记录 - 设置 phase 及计数（作为无 Sheet 记录时的回退值）
            file_progress.file_phase = record.file_phase;
            file_progress.total_rows = record.total_rows;
            file_progress.success_count = record.success_count;
            file_progress.error_count = record.error_count;
        }
    }

    // 若文件存在 Sheet 级别记录，用各 Sheet 之和重新计算文件统计，
    // 避免与文件级别记录重复计数（两者均存储了相同的成功行数）
    for file_progress in files_map.values_mut() {
        if !file_progress.sheets.is_empty() {
            file_progress.total_rows = file_progress.sheets.iter().map(|s| s.total_rows).sum();
            file_progress.success_count = file_progress.sheets.iter().map(|s| s.success_count).sum();
            file_progress.error_count = file_progress.sheets.iter().map(|s| s.error_count).sum();
        }

        // 兜底修正：文件已完成时，将残留中间态的 sheet（空 sheet 未能正确更新状态）
        // 标记为 done，避免已完成任务中出现"AI 识别中"/"导入中"等过时状态
        if file_progress.file_phase == "done" {
            for sheet in file_progress.sheets.iter_mut() {
                if sheet.sheet_phase == "ai_analyzing" || sheet.sheet_phase == "importing" {
                    sheet.sheet_phase = "done".to_string();
                }
            }
        }
    }

    // 转换为 Vec，保持原始文件顺序（通过插入顺序）
    let files: Vec<FileProgressResponse> = files_map.into_values().collect();

    Ok(FullTaskProgressResponse {
        task_id,
        files,
    })
}

/// 更新或插入文件进度（供 processing.rs 内部调用）
pub async fn upsert_file_progress(
    db: &Arc<DatabaseConnection>,
    task_id: &str,
    file_name: &str,
    sheet_name: Option<&str>,
    file_phase: Option<&str>,
    sheet_phase: Option<&str>,
    ai_confidence: Option<f32>,
    mapping_count: Option<i32>,
    success_count: Option<i32>,
    error_count: Option<i32>,
    total_rows: Option<i32>,
    error_message: Option<&str>,
) -> Result<(), String> {
    let now = chrono::Utc::now();

    // 查找现有记录
    let existing = if let Some(sheet) = sheet_name {
        TaskFileProgress::find()
            .filter(task_file_progress::Column::TaskId.eq(task_id))
            .filter(task_file_progress::Column::FileName.eq(file_name))
            .filter(task_file_progress::Column::SheetName.eq(sheet))
            .one(db.as_ref())
            .await
            .map_err(|e| format!("数据库错误: {}", e))?
    } else {
        TaskFileProgress::find()
            .filter(task_file_progress::Column::TaskId.eq(task_id))
            .filter(task_file_progress::Column::FileName.eq(file_name))
            .filter(task_file_progress::Column::SheetName.is_null())
            .one(db.as_ref())
            .await
            .map_err(|e| format!("数据库错误: {}", e))?
    };

    if let Some(model) = existing {
        // 更新现有记录
        let mut active: task_file_progress::ActiveModel = model.into();

        if let Some(phase) = file_phase {
            active.file_phase = Set(phase.to_string());
        }
        if let Some(phase) = sheet_phase {
            active.sheet_phase = Set(Some(phase.to_string()));
        }
        if ai_confidence.is_some() {
            active.ai_confidence = Set(ai_confidence);
        }
        if mapping_count.is_some() {
            active.mapping_count = Set(mapping_count);
        }
        if let Some(count) = success_count {
            active.success_count = Set(count);
        }
        if let Some(count) = error_count {
            active.error_count = Set(count);
        }
        if let Some(count) = total_rows {
            active.total_rows = Set(count);
        }
        if error_message.is_some() {
            active.error_message = Set(error_message.map(|s| s.to_string()));
        }
        active.updated_at = Set(Some(now));

        active.update(db.as_ref()).await.map_err(|e| format!("数据库错误: {}", e))?;
    } else {
        // 创建新记录
        let new_record = task_file_progress::ActiveModel {
            task_id: Set(task_id.to_string()),
            file_name: Set(file_name.to_string()),
            file_phase: Set(file_phase.unwrap_or("waiting").to_string()),
            sheet_name: Set(sheet_name.map(|s| s.to_string())),
            sheet_phase: Set(sheet_phase.map(|s| s.to_string())),
            ai_confidence: Set(ai_confidence),
            mapping_count: Set(mapping_count),
            success_count: Set(success_count.unwrap_or(0)),
            error_count: Set(error_count.unwrap_or(0)),
            total_rows: Set(total_rows.unwrap_or(0)),
            error_message: Set(error_message.map(|s| s.to_string())),
            created_at: Set(now),
            updated_at: Set(None),
            ..Default::default()
        };

        new_record.insert(db.as_ref()).await.map_err(|e| format!("数据库错误: {}", e))?;
    }

    Ok(())
}

/// 重置任务（可选删除已导入记录）
#[tauri::command]
pub async fn reset_processing_task(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    task_id: String,
    delete_records: bool,
) -> Result<TaskResponse, String> {
    let now = chrono::Utc::now();

    // 获取任务
    let task = ProcessingTask::find_by_id(&task_id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("任务 {} 不存在", task_id))?;

    // 如果需要删除记录
    if delete_records {
        use crate::backend::infrastructure::persistence::models::record;

        // 删除该批次的所有记录
        let batch_number = task.batch_number.clone();
        if let Some(batch) = batch_number {
            let delete_result = ProjectRecord::delete_many()
                .filter(record::Column::BatchNumber.eq(&batch))
                .exec(db.inner().as_ref())
                .await
                .map_err(|e| format!("删除记录失败: {}", e))?;
            tracing::info!("Deleted {} records for batch {}", delete_result.rows_affected, batch);
        }
    }

    // 删除任务进度记录
    TaskFileProgress::delete_many()
        .filter(task_file_progress::Column::TaskId.eq(&task_id))
        .exec(db.inner().as_ref())
        .await
        .map_err(|e| format!("删除进度记录失败: {}", e))?;

    // 重置任务状态
    let mut active: task::ActiveModel = task.into();
    active.status = Set("pending".to_string());
    active.processed_files = Set(0);
    active.total_rows = Set(0);
    active.processed_rows = Set(0);
    active.success_count = Set(0);
    active.error_count = Set(0);
    active.updated_at = Set(Some(now));

    let result = active
        .update(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.into())
}

// ============ 导入撤回功能 ============

/// 撤回结果
#[derive(Debug, Serialize)]
pub struct RollbackResult {
    pub success: bool,
    pub deleted_count: u64,
    pub message: String,
}

/// 导入记录详情（以任务为单位，1个任务=1个文件）
#[derive(Debug, Serialize)]
pub struct BatchDetailResponse {
    pub batch_number: String,
    pub task_id: String,
    pub source_file: String,
    pub project_id: i32,
    pub created_at: String,
    pub status: String,
    pub total_records: i32,
}

/// 撤回整个导入（通过 batch_number 关联到 processing_tasks）
#[tauri::command]
pub async fn rollback_batch(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    batch_number: String,
) -> Result<RollbackResult, String> {
    use crate::backend::infrastructure::persistence::models::record;

    tracing::info!("Rolling back batch {} for project {}", batch_number, project_id);

    // 验证该 batch_number 存在且属于该项目（从 processing_tasks 验证）
    let _task = ProcessingTask::find()
        .filter(task::Column::BatchNumber.eq(&batch_number))
        .filter(task::Column::ProjectId.eq(project_id))
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("导入记录 {} 不存在或不属于项目 {}", batch_number, project_id))?;

    // 删除该 batch_number 的所有记录
    let delete_result = ProjectRecord::delete_many()
        .filter(record::Column::ProjectId.eq(project_id))
        .filter(record::Column::BatchNumber.eq(&batch_number))
        .exec(db.inner().as_ref())
        .await
        .map_err(|e| format!("删除记录失败: {}", e))?;

    let deleted_count = delete_result.rows_affected;

    tracing::info!("Rolled back batch {}, deleted {} records", batch_number, deleted_count);

    Ok(RollbackResult {
        success: true,
        deleted_count,
        message: format!("已撤回导入，删除了 {} 条记录", deleted_count),
    })
}

/// 获取项目的所有导入记录列表（以任务为单位，带实时记录数统计）
#[tauri::command]
pub async fn get_project_batches_with_stats(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
) -> Result<Vec<BatchDetailResponse>, String> {
    // 从 processing_tasks 获取所有任务（有 batch_number 的）
    let tasks = ProcessingTask::find()
        .filter(task::Column::ProjectId.eq(project_id))
        .filter(task::Column::BatchNumber.is_not_null())
        .order_by_desc(task::Column::CreatedAt)
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    let mut results = Vec::new();

    for task_model in tasks {
        let batch_number = match task_model.batch_number.clone() {
            Some(bn) => bn,
            None => continue,
        };

        // 从 source_files JSON 提取第一个文件名作为显示名称
        let source_file = task_model.source_files
            .as_deref()
            .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
            .and_then(|v| v.into_iter().next())
            .unwrap_or_else(|| batch_number.clone());

        // 查询实时记录数
        let count_sql = "SELECT COUNT(*) FROM project_records WHERE project_id = ? AND batch_number = ?";
        let actual_count: i64 = db.inner().as_ref()
            .query_one(Statement::from_sql_and_values(
                db.inner().as_ref().get_database_backend(),
                count_sql,
                [project_id.into(), batch_number.clone().into()],
            ))
            .await
            .map_err(|e| format!("数据库错误: {}", e))?
            .map(|r| r.try_get_by_index::<i64>(0).unwrap_or(0))
            .unwrap_or(0);

        let status = if actual_count > 0 {
            task_model.status.clone()
        } else if task_model.status == "completed" {
            "rolled_back".to_string()
        } else {
            task_model.status.clone()
        };

        results.push(BatchDetailResponse {
            batch_number,
            task_id: task_model.id,
            source_file,
            project_id: task_model.project_id,
            created_at: task_model.created_at.to_rfc3339(),
            status,
            total_records: actual_count as i32,
        });
    }

    Ok(results)
}
