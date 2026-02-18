// 记录管理 Tauri Commands
//
// 使用 JSON 统一存储方案，data 列以 field_id 为 key
// 如 {"3": "张三", "5": "13800138000"}

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    QueryFilter, Set, Statement, PaginatorTrait,
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

use crate::backend::infrastructure::persistence::models::{record, ProjectRecord};

// ============ 响应结构 ============

#[derive(Debug, Serialize)]
pub struct RecordResponse {
    pub id: i32,
    pub project_id: i32,
    pub data: JsonValue,
    pub source_file: Option<String>,
    pub source_sheet: Option<String>,
    pub row_number: Option<i32>,
    pub batch_number: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QueryRecordsResponse {
    pub records: Vec<RecordResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

impl From<record::Model> for RecordResponse {
    fn from(m: record::Model) -> Self {
        let data: JsonValue = serde_json::from_str(&m.data)
            .unwrap_or(JsonValue::Object(Default::default()));
        Self {
            id: m.id,
            project_id: m.project_id,
            data,
            source_file: m.source_file,
            source_sheet: m.source_sheet,
            row_number: m.row_number,
            batch_number: m.batch_number,
            status: m.status,
            error_message: m.error_message,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

// ============ Tauri Commands ============

/// 插入单条记录
#[tauri::command]
pub async fn insert_record(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    data: JsonValue,
    source_file: Option<String>,
    source_sheet: Option<String>,
    row_number: Option<i32>,
    batch_number: Option<String>,
    status: Option<String>,
    error_message: Option<String>,
) -> Result<RecordResponse, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let data_str = serde_json::to_string(&data)
        .map_err(|e| format!("JSON 序列化错误: {}", e))?;

    let new_record = record::ActiveModel {
        project_id: Set(project_id),
        data: Set(data_str),
        source_file: Set(source_file),
        source_sheet: Set(source_sheet),
        row_number: Set(row_number),
        batch_number: Set(batch_number),
        status: Set(status.unwrap_or_else(|| "success".to_string())),
        error_message: Set(error_message),
        created_at: Set(now),
        updated_at: Set(None),
        ..Default::default()
    };

    let result = new_record
        .insert(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.into())
}

/// 批量插入记录
#[tauri::command]
pub async fn insert_records_batch(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    records: Vec<JsonValue>,
    source_file: Option<String>,
    source_sheet: Option<String>,
    batch_number: Option<String>,
) -> Result<u64, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let mut count: u64 = 0;

    for (idx, data) in records.iter().enumerate() {
        let data_str = serde_json::to_string(data)
            .map_err(|e| format!("JSON 序列化错误 (行 {}): {}", idx, e))?;

        let new_record = record::ActiveModel {
            project_id: Set(project_id),
            data: Set(data_str),
            source_file: Set(source_file.clone()),
            source_sheet: Set(source_sheet.clone()),
            row_number: Set(Some(idx as i32 + 1)),
            batch_number: Set(batch_number.clone()),
            status: Set("success".to_string()),
            error_message: Set(None),
            created_at: Set(now.clone()),
            updated_at: Set(None),
            ..Default::default()
        };

        new_record
            .insert(db.inner().as_ref())
            .await
            .map_err(|e| format!("数据库错误 (行 {}): {}", idx, e))?;

        count += 1;
    }

    Ok(count)
}

/// 分页查询记录（支持 json_extract 过滤）
#[tauri::command]
pub async fn query_records(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    page: Option<u64>,
    page_size: Option<u64>,
    batch_number: Option<String>,
    status: Option<String>,
    filters: Option<HashMap<String, String>>,
) -> Result<QueryRecordsResponse, String> {
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(20).min(500);
    let offset = (page - 1) * page_size;

    // 构建 WHERE 子句
    let mut conditions = vec!["project_id = ?".to_string()];
    let mut params: Vec<String> = vec![project_id.to_string()];

    if let Some(bn) = &batch_number {
        conditions.push("batch_number = ?".to_string());
        params.push(bn.clone());
    }
    if let Some(st) = &status {
        conditions.push("status = ?".to_string());
        params.push(st.clone());
    }
    if let Some(f) = &filters {
        for (field_id, value) in f {
            conditions.push(format!("json_extract(data, '$.{}') = ?", field_id));
            params.push(value.clone());
        }
    }

    let where_clause = conditions.join(" AND ");

    // 查询总数
    let count_sql = format!("SELECT COUNT(*) as cnt FROM project_records WHERE {}", where_clause);
    let count_result = db.inner().as_ref()
        .query_one(Statement::from_sql_and_values(
            db.inner().as_ref().get_database_backend(),
            &count_sql,
            params.iter().map(|p| p.clone().into()).collect::<Vec<sea_orm::Value>>(),
        ))
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    let total: u64 = count_result
        .map(|r| r.try_get_by_index::<i64>(0).unwrap_or(0) as u64)
        .unwrap_or(0);

    // 查询数据
    let query_sql = format!(
        "SELECT * FROM project_records WHERE {} ORDER BY id DESC LIMIT ? OFFSET ?",
        where_clause
    );
    let mut query_params: Vec<sea_orm::Value> = params.iter().map(|p| p.clone().into()).collect();
    query_params.push((page_size as i64).into());
    query_params.push((offset as i64).into());

    let rows = db.inner().as_ref()
        .query_all(Statement::from_sql_and_values(
            db.inner().as_ref().get_database_backend(),
            &query_sql,
            query_params,
        ))
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    let records: Vec<RecordResponse> = rows.iter().map(|row| {
        let data_str: String = row.try_get_by::<String, _>("data").unwrap_or_default();
        let data: JsonValue = serde_json::from_str(&data_str)
            .unwrap_or(JsonValue::Object(Default::default()));
        RecordResponse {
            id: row.try_get_by::<i32, _>("id").unwrap_or(0),
            project_id: row.try_get_by::<i32, _>("project_id").unwrap_or(0),
            data,
            source_file: row.try_get_by::<Option<String>, _>("source_file").unwrap_or(None),
            source_sheet: row.try_get_by::<Option<String>, _>("source_sheet").unwrap_or(None),
            row_number: row.try_get_by::<Option<i32>, _>("row_number").unwrap_or(None),
            batch_number: row.try_get_by::<Option<String>, _>("batch_number").unwrap_or(None),
            status: row.try_get_by::<String, _>("status").unwrap_or_default(),
            error_message: row.try_get_by::<Option<String>, _>("error_message").unwrap_or(None),
            created_at: row.try_get_by::<String, _>("created_at").unwrap_or_default(),
            updated_at: row.try_get_by::<Option<String>, _>("updated_at").unwrap_or(None),
        }
    }).collect();

    Ok(QueryRecordsResponse {
        records,
        total,
        page,
        page_size,
    })
}

/// 获取单条记录
#[tauri::command]
pub async fn get_record(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
) -> Result<RecordResponse, String> {
    let record = ProjectRecord::find_by_id(id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("记录 {} 不存在", id))?;

    Ok(record.into())
}

/// 更新记录的 data
#[tauri::command]
pub async fn update_record(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
    data: JsonValue,
) -> Result<RecordResponse, String> {
    let record = ProjectRecord::find_by_id(id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("记录 {} 不存在", id))?;

    let data_str = serde_json::to_string(&data)
        .map_err(|e| format!("JSON 序列化错误: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();

    let mut active: record::ActiveModel = record.into();
    active.data = Set(data_str);
    active.updated_at = Set(Some(now));

    let result = active
        .update(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.into())
}

/// 删除单条记录
#[tauri::command]
pub async fn delete_record(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
) -> Result<(), String> {
    ProjectRecord::delete_by_id(id)
        .exec(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(())
}

/// 删除项目所有记录
#[tauri::command]
pub async fn delete_project_records(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
) -> Result<u64, String> {
    let result = ProjectRecord::delete_many()
        .filter(record::Column::ProjectId.eq(project_id))
        .exec(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.rows_affected)
}

/// 获取项目记录数
#[tauri::command]
pub async fn get_record_count(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    status: Option<String>,
) -> Result<u64, String> {
    let mut query = ProjectRecord::find()
        .filter(record::Column::ProjectId.eq(project_id));

    if let Some(st) = status {
        query = query.filter(record::Column::Status.eq(st));
    }

    let count = query
        .count(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(count)
}

/// 去重检查：根据指定字段值检查是否存在重复记录
#[tauri::command]
pub async fn check_duplicate(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    dedup_values: HashMap<String, String>,
) -> Result<Option<i32>, String> {
    if dedup_values.is_empty() {
        return Ok(None);
    }

    let mut conditions = vec!["project_id = ?".to_string()];
    let mut params: Vec<sea_orm::Value> = vec![project_id.into()];

    for (field_id, value) in &dedup_values {
        conditions.push(format!("json_extract(data, '$.{}') = ?", field_id));
        params.push(value.clone().into());
    }

    let sql = format!(
        "SELECT id FROM project_records WHERE {} LIMIT 1",
        conditions.join(" AND ")
    );

    let result = db.inner().as_ref()
        .query_one(Statement::from_sql_and_values(
            db.inner().as_ref().get_database_backend(),
            &sql,
            params,
        ))
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.and_then(|r| r.try_get_by_index::<i32>(0).ok()))
}
