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
    pub raw_data: Option<String>,
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
            raw_data: m.raw_data,
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

/// 分页查询记录（支持搜索和 json_extract 过滤）
#[tauri::command]
pub async fn query_records(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    page: Option<u64>,
    page_size: Option<u64>,
    batch_number: Option<String>,
    status: Option<String>,
    search: Option<String>,
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
    // 搜索：在 JSON data 字段中模糊匹配
    if let Some(s) = &search {
        if !s.trim().is_empty() {
            conditions.push("data LIKE ?".to_string());
            params.push(format!("%{}%", s.trim()));
        }
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
        let raw_data: Option<String> = row.try_get_by::<Option<String>, _>("raw_data").unwrap_or(None);
        RecordResponse {
            id: row.try_get_by::<i32, _>("id").unwrap_or(0),
            project_id: row.try_get_by::<i32, _>("project_id").unwrap_or(0),
            data,
            raw_data,
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

// ============ 高级筛选相关 ============

/// 筛选运算符
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Eq,
    Neq,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Gt,
    Lt,
    Gte,
    Lte,
    Between,
    IsEmpty,
    IsNotEmpty,
}

/// 单个筛选条件
#[derive(Debug, serde::Deserialize)]
pub struct FilterCondition {
    pub field: String,
    pub operator: FilterOperator,
    pub value: Option<JsonValue>,  // 单值或范围值 [start, end]
}

/// 高级筛选请求
#[derive(Debug, serde::Deserialize)]
pub struct AdvancedFilterRequest {
    pub search: Option<String>,
    pub conditions: Vec<FilterCondition>,
    pub source_file: Option<String>,
    pub source_sheet: Option<String>,
    pub batch_number: Option<String>,
    pub status: Option<String>,
    pub conjunction: Option<String>,  // "and" 或 "or"，默认 "and"
}

/// 高级记录查询
#[tauri::command]
pub async fn query_records_advanced(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    page: Option<u64>,
    page_size: Option<u64>,
    filter: AdvancedFilterRequest,
) -> Result<QueryRecordsResponse, String> {
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(20).min(500);
    let offset = (page - 1) * page_size;

    let (where_clause, params) = build_where_for_filter(project_id, &filter);

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
        let raw_data: Option<String> = row.try_get_by::<Option<String>, _>("raw_data").unwrap_or(None);
        RecordResponse {
            id: row.try_get_by::<i32, _>("id").unwrap_or(0),
            project_id: row.try_get_by::<i32, _>("project_id").unwrap_or(0),
            data,
            raw_data,
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

/// 获取字段唯一值（用于下拉筛选）
#[tauri::command]
pub async fn get_field_distinct_values(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    field_id: String,
    search: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<String>, String> {
    let limit = limit.unwrap_or(100).min(500);

    let sql = if let Some(s) = &search {
        if !s.trim().is_empty() {
            format!(
                "SELECT DISTINCT json_extract(data, '$.{}') as value FROM project_records \
                 WHERE project_id = ? AND json_extract(data, '$.{}') LIKE ? \
                 ORDER BY value LIMIT ?",
                field_id, field_id
            )
        } else {
            format!(
                "SELECT DISTINCT json_extract(data, '$.{}') as value FROM project_records \
                 WHERE project_id = ? ORDER BY value LIMIT ?",
                field_id
            )
        }
    } else {
        format!(
            "SELECT DISTINCT json_extract(data, '$.{}') as value FROM project_records \
             WHERE project_id = ? ORDER BY value LIMIT ?",
            field_id
        )
    };

    let params: Vec<sea_orm::Value> = if let Some(s) = &search {
        if !s.trim().is_empty() {
            vec![project_id.into(), format!("%{}%", s.trim()).into(), (limit as i64).into()]
        } else {
            vec![project_id.into(), (limit as i64).into()]
        }
    } else {
        vec![project_id.into(), (limit as i64).into()]
    };

    let rows = db.inner().as_ref()
        .query_all(Statement::from_sql_and_values(
            db.inner().as_ref().get_database_backend(),
            &sql,
            params,
        ))
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    let values: Vec<String> = rows.iter()
        .filter_map(|r| r.try_get_by::<Option<String>, _>("value").ok().flatten())
        .filter(|v| v != "null" && !v.is_empty())
        .collect();

    Ok(values)
}

/// 来源文件信息
#[derive(Debug, Serialize)]
pub struct SourceFileInfo {
    pub source_file: String,
    pub record_count: u64,
}

/// 获取来源文件列表
#[tauri::command]
pub async fn get_source_files(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
) -> Result<Vec<SourceFileInfo>, String> {
    let sql = format!(
        "SELECT source_file, COUNT(*) as cnt FROM project_records \
         WHERE project_id = ? AND source_file IS NOT NULL \
         GROUP BY source_file ORDER BY source_file"
    );

    let rows = db.inner().as_ref()
        .query_all(Statement::from_sql_and_values(
            db.inner().as_ref().get_database_backend(),
            &sql,
            vec![project_id.into()],
        ))
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    let files: Vec<SourceFileInfo> = rows.iter()
        .filter_map(|r| {
            let source_file = r.try_get_by::<Option<String>, _>("source_file").ok().flatten()?;
            let record_count = r.try_get_by::<i64, _>("cnt").unwrap_or(0) as u64;
            Some(SourceFileInfo { source_file, record_count })
        })
        .collect();

    Ok(files)
}

/// 辅助函数：JsonValue 转字符串
fn json_value_to_string(v: &JsonValue) -> String {
    match v {
        JsonValue::String(s) => s.clone(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => b.to_string(),
        _ => v.to_string(),
    }
}

// ============ xlsx 导出辅助 ============

/// 辅助函数：根据高级筛选请求构建 WHERE 子句和参数列表
fn build_where_for_filter(project_id: i32, filter: &AdvancedFilterRequest) -> (String, Vec<String>) {
    let conjunction = filter.conjunction.as_deref().unwrap_or("and");
    let joiner = if conjunction == "or" { " OR " } else { " AND " };

    let mut conditions = vec!["project_id = ?".to_string()];
    let mut params: Vec<String> = vec![project_id.to_string()];

    if let Some(s) = &filter.search {
        if !s.trim().is_empty() {
            conditions.push("data LIKE ?".to_string());
            params.push(format!("%{}%", s.trim()));
        }
    }
    if let Some(sf) = &filter.source_file {
        conditions.push("source_file = ?".to_string());
        params.push(sf.clone());
    }
    if let Some(ss) = &filter.source_sheet {
        conditions.push("source_sheet = ?".to_string());
        params.push(ss.clone());
    }
    if let Some(bn) = &filter.batch_number {
        conditions.push("batch_number = ?".to_string());
        params.push(bn.clone());
    }
    if let Some(st) = &filter.status {
        conditions.push("status = ?".to_string());
        params.push(st.clone());
    }

    let mut field_conditions: Vec<String> = Vec::new();
    for cond in &filter.conditions {
        let field_expr = format!("json_extract(data, '$.{}')", cond.field);
        match &cond.operator {
            FilterOperator::Eq => {
                if let Some(v) = &cond.value {
                    field_conditions.push(format!("{} = ?", field_expr));
                    params.push(json_value_to_string(v));
                }
            }
            FilterOperator::Neq => {
                if let Some(v) = &cond.value {
                    field_conditions.push(format!("{} != ?", field_expr));
                    params.push(json_value_to_string(v));
                }
            }
            FilterOperator::Contains => {
                if let Some(v) = &cond.value {
                    field_conditions.push(format!("{} LIKE ?", field_expr));
                    params.push(format!("%{}%", json_value_to_string(v)));
                }
            }
            FilterOperator::NotContains => {
                if let Some(v) = &cond.value {
                    field_conditions.push(format!("{} NOT LIKE ?", field_expr));
                    params.push(format!("%{}%", json_value_to_string(v)));
                }
            }
            FilterOperator::StartsWith => {
                if let Some(v) = &cond.value {
                    field_conditions.push(format!("{} LIKE ?", field_expr));
                    params.push(format!("{}%", json_value_to_string(v)));
                }
            }
            FilterOperator::EndsWith => {
                if let Some(v) = &cond.value {
                    field_conditions.push(format!("{} LIKE ?", field_expr));
                    params.push(format!("%{}", json_value_to_string(v)));
                }
            }
            FilterOperator::Gt => {
                if let Some(v) = &cond.value {
                    field_conditions.push(format!("{} > ?", field_expr));
                    params.push(json_value_to_string(v));
                }
            }
            FilterOperator::Lt => {
                if let Some(v) = &cond.value {
                    field_conditions.push(format!("{} < ?", field_expr));
                    params.push(json_value_to_string(v));
                }
            }
            FilterOperator::Gte => {
                if let Some(v) = &cond.value {
                    field_conditions.push(format!("{} >= ?", field_expr));
                    params.push(json_value_to_string(v));
                }
            }
            FilterOperator::Lte => {
                if let Some(v) = &cond.value {
                    field_conditions.push(format!("{} <= ?", field_expr));
                    params.push(json_value_to_string(v));
                }
            }
            FilterOperator::Between => {
                if let Some(v) = &cond.value {
                    if let Some(arr) = v.as_array() {
                        if arr.len() >= 2 {
                            field_conditions.push(format!("{} BETWEEN ? AND ?", field_expr));
                            params.push(json_value_to_string(&arr[0]));
                            params.push(json_value_to_string(&arr[1]));
                        }
                    }
                }
            }
            FilterOperator::IsEmpty => {
                field_conditions.push(format!(
                    "({} IS NULL OR {} = '' OR {} = 'null')",
                    field_expr, field_expr, field_expr
                ));
            }
            FilterOperator::IsNotEmpty => {
                field_conditions.push(format!(
                    "({} IS NOT NULL AND {} != '' AND {} != 'null')",
                    field_expr, field_expr, field_expr
                ));
            }
        }
    }

    if !field_conditions.is_empty() {
        conditions.push(format!("({})", field_conditions.join(joiner)));
    }

    (conditions.join(" AND "), params)
}

/// 辅助函数：将 ISO 时间字符串格式化为本地时间显示（YYYY-MM-DD HH:MM）
fn format_xlsx_time(iso_str: &str) -> String {
    if iso_str.is_empty() {
        return String::new();
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso_str) {
        let local = dt.with_timezone(&chrono::Local);
        return local.format("%Y-%m-%d %H:%M").to_string();
    }
    iso_str.to_string()
}

/// 导出记录为 xlsx 文件并保存到指定路径
///
/// - filter=None：导出所有成功记录（不受筛选影响）
/// - filter=Some(f)：导出匹配筛选条件的记录
/// - page/page_size=Some：仅导出对应分页（当前页）
/// - page/page_size=None：导出所有匹配记录（无分页限制）
///
/// 返回实际导出的行数
#[tauri::command]
pub async fn export_records_xlsx(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    file_path: String,
    field_ids: Vec<String>,
    field_labels: Vec<String>,
    include_import_time: bool,
    include_source_file: bool,
    filter: Option<AdvancedFilterRequest>,
    page: Option<u64>,
    page_size: Option<u64>,
) -> Result<u64, String> {
    use rust_xlsxwriter::{Format, Workbook};

    // 构建 WHERE 子句
    let (where_clause, params) = match &filter {
        Some(f) => build_where_for_filter(project_id, f),
        None => (
            "project_id = ? AND status = ?".to_string(),
            vec![project_id.to_string(), "success".to_string()],
        ),
    };

    // 构建查询 SQL（含可选分页）
    let mut query_params: Vec<sea_orm::Value> = params.iter().map(|p| p.clone().into()).collect();
    let query_sql = match (page, page_size) {
        (Some(p), Some(ps)) => {
            let offset = (p.max(1) - 1) * ps;
            query_params.push((ps as i64).into());
            query_params.push((offset as i64).into());
            format!(
                "SELECT * FROM project_records WHERE {} ORDER BY id DESC LIMIT ? OFFSET ?",
                where_clause
            )
        }
        _ => format!(
            "SELECT * FROM project_records WHERE {} ORDER BY id DESC",
            where_clause
        ),
    };

    let rows = db
        .inner()
        .as_ref()
        .query_all(Statement::from_sql_and_values(
            db.inner().as_ref().get_database_backend(),
            &query_sql,
            query_params,
        ))
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    // 创建 xlsx 工作簿
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let header_format = Format::new().set_bold();

    // 写表头
    let mut col: u16 = 0;
    for label in &field_labels {
        worksheet
            .write_with_format(0, col, label.as_str(), &header_format)
            .map_err(|e| format!("xlsx 写入错误: {}", e))?;
        col += 1;
    }
    if include_import_time {
        worksheet
            .write_with_format(0, col, "导入时间", &header_format)
            .map_err(|e| format!("xlsx 写入错误: {}", e))?;
        col += 1;
    }
    if include_source_file {
        worksheet
            .write_with_format(0, col, "来源文件", &header_format)
            .map_err(|e| format!("xlsx 写入错误: {}", e))?;
    }

    // 写数据行
    for (idx, row) in rows.iter().enumerate() {
        let row_num = (idx + 1) as u32;
        let data_str: String = row.try_get_by::<String, _>("data").unwrap_or_default();
        let data: JsonValue = serde_json::from_str(&data_str)
            .unwrap_or(JsonValue::Object(Default::default()));
        let created_at: String = row
            .try_get_by::<String, _>("created_at")
            .unwrap_or_default();
        let source_file: Option<String> = row
            .try_get_by::<Option<String>, _>("source_file")
            .unwrap_or(None);
        let source_sheet: Option<String> = row
            .try_get_by::<Option<String>, _>("source_sheet")
            .unwrap_or(None);

        let mut col: u16 = 0;
        for field_id in &field_ids {
            let value = data
                .get(field_id)
                .map(|v| json_value_to_string(v))
                .unwrap_or_default();
            worksheet
                .write(row_num, col, value.as_str())
                .map_err(|e| format!("xlsx 写入错误: {}", e))?;
            col += 1;
        }
        if include_import_time {
            let time_str = format_xlsx_time(&created_at);
            worksheet
                .write(row_num, col, time_str.as_str())
                .map_err(|e| format!("xlsx 写入错误: {}", e))?;
            col += 1;
        }
        if include_source_file {
            let src = match (source_file, source_sheet) {
                (Some(f), Some(s)) => format!("{} / {}", f, s),
                (Some(f), None) => f,
                _ => String::new(),
            };
            worksheet
                .write(row_num, col, src.as_str())
                .map_err(|e| format!("xlsx 写入错误: {}", e))?;
        }
    }

    // 保存文件
    workbook
        .save(&file_path)
        .map_err(|e| format!("保存 xlsx 失败: {}", e))?;

    Ok(rows.len() as u64)
}
