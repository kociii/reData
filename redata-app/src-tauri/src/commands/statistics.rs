// 项目统计 Tauri Commands
//
// 获取项目的统计数据，包括记录数、任务数、成功率等

use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Statement,
};
use serde::Serialize;
use std::sync::Arc;

use crate::backend::infrastructure::persistence::models::{record, task, ProjectRecord, ProcessingTask};

// ============ 响应结构 ============

/// 项目统计数据
#[derive(Debug, Serialize)]
pub struct ProjectStatistics {
    /// 总记录数
    pub total_records: u64,
    /// 今日新增记录数
    pub today_records: u64,
    /// 本周新增记录数（最近 7 天）
    pub week_records: u64,
    /// 本月新增记录数（最近 30 天）
    pub month_records: u64,
    /// 处理任务总数
    pub total_tasks: u64,
    /// 成功任务数
    pub success_tasks: u64,
    /// 成功率（百分比，0-100）
    pub success_rate: f64,
    /// 最后处理时间
    pub last_processed_at: Option<String>,
}

// ============ Tauri Commands ============

/// 获取项目统计数据
#[tauri::command]
pub async fn get_project_statistics(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
) -> Result<ProjectStatistics, String> {
    let conn = db.inner().as_ref();

    // 1. 总记录数
    let total_records = ProjectRecord::find()
        .filter(record::Column::ProjectId.eq(project_id))
        .count(conn)
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    // 2. 今日新增记录数
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let today_sql = format!(
        "SELECT COUNT(*) as cnt FROM project_records WHERE project_id = ? AND DATE(created_at) = DATE('{}')",
        today
    );
    let today_result = conn
        .query_one(Statement::from_sql_and_values(
            conn.get_database_backend(),
            &today_sql,
            vec![project_id.into()],
        ))
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;
    let today_records = today_result
        .map(|r| r.try_get_by_index::<i64>(0).unwrap_or(0) as u64)
        .unwrap_or(0);

    // 3. 本周新增记录数（最近 7 天）
    let week_ago = (chrono::Utc::now() - chrono::Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();
    let week_sql = format!(
        "SELECT COUNT(*) as cnt FROM project_records WHERE project_id = ? AND DATE(created_at) >= DATE('{}')",
        week_ago
    );
    let week_result = conn
        .query_one(Statement::from_sql_and_values(
            conn.get_database_backend(),
            &week_sql,
            vec![project_id.into()],
        ))
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;
    let week_records = week_result
        .map(|r| r.try_get_by_index::<i64>(0).unwrap_or(0) as u64)
        .unwrap_or(0);

    // 4. 本月新增记录数（最近 30 天）
    let month_ago = (chrono::Utc::now() - chrono::Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();
    let month_sql = format!(
        "SELECT COUNT(*) as cnt FROM project_records WHERE project_id = ? AND DATE(created_at) >= DATE('{}')",
        month_ago
    );
    let month_result = conn
        .query_one(Statement::from_sql_and_values(
            conn.get_database_backend(),
            &month_sql,
            vec![project_id.into()],
        ))
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;
    let month_records = month_result
        .map(|r| r.try_get_by_index::<i64>(0).unwrap_or(0) as u64)
        .unwrap_or(0);

    // 5. 任务统计
    let total_tasks = ProcessingTask::find()
        .filter(task::Column::ProjectId.eq(project_id))
        .count(conn)
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    let success_tasks = ProcessingTask::find()
        .filter(task::Column::ProjectId.eq(project_id))
        .filter(task::Column::Status.eq("completed"))
        .count(conn)
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    let success_rate = if total_tasks > 0 {
        (success_tasks as f64 / total_tasks as f64) * 100.0
    } else {
        0.0
    };

    // 6. 最后处理时间
    let last_sql = "SELECT MAX(created_at) as last_time FROM project_records WHERE project_id = ?";
    let last_result = conn
        .query_one(Statement::from_sql_and_values(
            conn.get_database_backend(),
            last_sql,
            vec![project_id.into()],
        ))
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;
    let last_processed_at = last_result.and_then(|r| {
        r.try_get_by_index::<String>(0).ok()
    });

    Ok(ProjectStatistics {
        total_records,
        today_records,
        week_records,
        month_records,
        total_tasks,
        success_tasks,
        success_rate: (success_rate * 10.0).round() / 10.0, // 保留一位小数
        last_processed_at,
    })
}
