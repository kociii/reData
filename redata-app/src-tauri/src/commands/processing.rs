// 数据处理 Tauri Commands
//
// 核心处理流程：AI 列映射 + 本地验证导入
// 使用 Tauri 事件系统推送进度

use calamine::{open_workbook_auto, Reader, Data};
use regex::Regex;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, Set, Statement,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

use crate::backend::infrastructure::{
    config::decrypt,
    persistence::models::{
        task, ProcessingTask, field,
        AiConfig as AiConfigModel, Project, record,
    },
};
use field::Model as FieldModel;
use super::ai_utils::{call_ai, extract_json};
use super::ai_service::FieldDefinition;

// ============ 任务控制 ============

struct TaskControl {
    paused: AtomicBool,
    cancelled: AtomicBool,
}

static ACTIVE_TASKS: LazyLock<RwLock<HashMap<String, Arc<TaskControl>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

// ============ 事件结构 ============

#[derive(Debug, Clone, Serialize, Default)]
pub struct ProcessingEvent {
    pub event: String,
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_sheet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_row: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_rows: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processed_rows: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mappings: Option<HashMap<String, String>>,
}

impl ProcessingEvent {
    fn emit(&self, app: &AppHandle) {
        let _ = app.emit("processing-progress", self);
    }
}

// ============ 响应结构 ============

#[derive(Debug, Serialize)]
pub struct StartProcessingResponse {
    pub task_id: String,
    pub batch_number: String,
    pub project_id: i32,
    pub status: String,
}

// ============ 辅助函数 ============

fn data_to_string(data: &Data) -> String {
    match data {
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            if *f == (*f as i64) as f64 {
                (*f as i64).to_string()
            } else {
                f.to_string()
            }
        }
        Data::String(s) => s.clone(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERR:{:?}", e),
        Data::Empty => String::new(),
    }
}

fn validate_value(value: &str, validation_rule: Option<&str>) -> bool {
    if value.trim().is_empty() {
        return true; // 空值通过（由 required 字段处理）
    }
    if let Some(rule) = validation_rule {
        if let Ok(re) = Regex::new(rule) {
            return re.is_match(value);
        }
    }
    true
}

/// 根据字段类型清理数据值
///
/// 清理规则：
/// - 通用：去除首尾空格、换行符、制表符
/// - phone: 仅保留数字和 + 号
/// - email: 去除空格、换行，转小写
/// - text/其他: 压缩连续空白为单个空格
fn clean_value(value: &str, field_type: &str) -> String {
    // 第一步：通用清理 - 去除首尾空白和控制字符
    let mut cleaned = value
        .chars()
        .map(|c| match c {
            '\r' | '\n' | '\t' => ' ',  // 换行、制表符转为空格
            c if c.is_control() => ' ', // 其他控制字符转为空格
            c => c,
        })
        .collect::<String>();

    // 根据字段类型进行特定清理
    match field_type {
        "phone" => {
            // 手机号：仅保留数字和 + 号
            cleaned = cleaned
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '+')
                .collect();
        }
        "email" => {
            // 邮箱：去除所有空格，转小写
            cleaned = cleaned.chars().filter(|c| !c.is_whitespace()).collect();
            cleaned = cleaned.to_lowercase();
        }
        "number" | "id_card" => {
            // 数字、身份证：仅保留数字和字母
            cleaned = cleaned
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect();
        }
        "date" => {
            // 日期：去除空格，保留数字、日期分隔符
            cleaned = cleaned
                .chars()
                .filter(|c| c.is_ascii_digit() || matches!(c, '-' | '/' | '.' | ':'))
                .collect();
        }
        _ => {
            // 默认文本类型：压缩连续空白为单个空格
            let mut result = String::new();
            let mut prev_space = false;
            for c in cleaned.chars() {
                if c.is_whitespace() {
                    if !prev_space {
                        result.push(' ');
                        prev_space = true;
                    }
                } else {
                    result.push(c);
                    prev_space = false;
                }
            }
            cleaned = result;
        }
    }

    // 最后再次 trim
    cleaned.trim().to_string()
}

// ============ Tauri Commands ============

/// 开始处理文件
#[tauri::command]
pub async fn start_processing(
    app: AppHandle,
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    file_paths: Vec<String>,
    ai_config_id: Option<i32>,
) -> Result<StartProcessingResponse, String> {
    // 获取数据库连接的克隆
    let db_conn = db.inner().clone();

    // 1. 验证项目
    let project = Project::find_by_id(project_id)
        .one(db_conn.as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("项目 {} 不存在", project_id))?;

    // 2. 获取字段定义
    let fields = field::Entity::find()
        .filter(field::Column::ProjectId.eq(project_id))
        .filter(field::Column::IsDeleted.eq(false))
        .order_by(field::Column::DisplayOrder, sea_orm::Order::Asc)
        .all(db_conn.as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    if fields.is_empty() {
        return Err("项目没有定义字段".to_string());
    }

    // 3. 获取 AI 配置
    let ai_config = if let Some(config_id) = ai_config_id {
        AiConfigModel::find_by_id(config_id)
            .one(db_conn.as_ref())
            .await
            .map_err(|e| format!("数据库错误: {}", e))?
            .ok_or_else(|| format!("AI 配置 {} 不存在", config_id))?
    } else {
        // 使用默认配置
        let configs = AiConfigModel::find()
            .all(db_conn.as_ref())
            .await
            .map_err(|e| format!("数据库错误: {}", e))?;
        configs.into_iter()
            .find(|c| c.is_default)
            .ok_or_else(|| "没有默认 AI 配置".to_string())?
    };

    // 4. 解密 API Key
    let api_key = decrypt(&ai_config.api_key)
        .map_err(|e| format!("解密失败: {}", e))?;

    // 5. 创建任务
    let task_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    // 生成 batch_number
    let date_str = now.format("%Y%m%d").to_string();
    let count = ProcessingTask::find()
        .filter(task::Column::ProjectId.eq(project_id))
        .filter(task::Column::BatchNumber.starts_with(&format!("BATCH_{}", date_str)))
        .all(db_conn.as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .len();
    let batch_number = format!("BATCH_{}_{:03}", date_str, count + 1);

    // 创建任务记录
    let new_task = task::ActiveModel {
        id: Set(task_id.clone()),
        project_id: Set(project_id),
        status: Set("processing".to_string()),
        total_files: Set(file_paths.len() as i32),
        processed_files: Set(0),
        total_rows: Set(0),
        processed_rows: Set(0),
        success_count: Set(0),
        error_count: Set(0),
        batch_number: Set(Some(batch_number.clone())),
        created_at: Set(now),
        updated_at: Set(None),
    };

    new_task
        .insert(db_conn.as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    // 6. 注册任务控制
    let control = Arc::new(TaskControl {
        paused: AtomicBool::new(false),
        cancelled: AtomicBool::new(false),
    });
    {
        let mut tasks = ACTIVE_TASKS.write().await;
        tasks.insert(task_id.clone(), control.clone());
    }

    // 7. 启动后台处理
    let app_for_spawn = app.clone();
    let project_clone = project.clone();
    let fields_clone = fields.clone();
    let api_url = ai_config.api_url.clone();
    let api_key_clone = api_key.clone();
    let model_name = ai_config.model_name.clone();
    let temperature = ai_config.temperature;
    let max_tokens = ai_config.max_tokens;
    let task_id_clone = task_id.clone();

    tokio::spawn(async move {
        let task_id_inner = task_id_clone;
        let result = process_files(
            app_for_spawn,
            db_conn.clone(),
            &task_id_inner,
            &project_clone,
            &fields_clone,
            &file_paths,
            &api_url,
            &api_key_clone,
            &model_name,
            temperature,
            max_tokens,
            control.clone(),
        ).await;

        // 清理任务控制
        {
            let mut tasks = ACTIVE_TASKS.write().await;
            tasks.remove(&task_id_inner);
        }

        // 更新最终状态
        if let Err(e) = result {
            let _ = update_task_error(&db_conn.clone(), &task_id_inner, &e).await;
            let event = ProcessingEvent {
                event: "error".to_string(),
                task_id: task_id_inner.clone(),
                message: Some(e),
                ..Default::default()
            };
            event.emit(&app);
        }
    });

    Ok(StartProcessingResponse {
        task_id,
        batch_number,
        project_id,
        status: "processing".to_string(),
    })
}

async fn update_task_error(db: &Arc<DatabaseConnection>, task_id: &str, _error: &str) -> Result<(), String> {
    let task = ProcessingTask::find_by_id(task_id)
        .one(db.as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    if let Some(task) = task {
        let mut active: task::ActiveModel = task.into();
        active.status = Set("error".to_string());
        active.updated_at = Set(Some(chrono::Utc::now()));
        active.error_count = Set(1);
        active.updated_at = Set(Some(chrono::Utc::now()));
        active.update(db.as_ref()).await.map_err(|e| format!("数据库错误: {}", e))?;
    }
    Ok(())
}

async fn process_files(
    app: AppHandle,
    db: Arc<DatabaseConnection>,
    task_id: &str,
    project: &crate::backend::infrastructure::persistence::models::project::Model,
    fields: &[FieldModel],
    file_paths: &[String],
    api_url: &str,
    api_key: &str,
    model_name: &str,
    temperature: f32,
    max_tokens: i32,
    control: Arc<TaskControl>,
) -> Result<(), String> {
    let mut total_rows = 0i32;
    let mut processed_rows = 0i32;
    let mut success_count = 0i32;
    let mut error_count = 0i32;

    // 获取去重字段
    let dedup_fields: Vec<i32> = if project.dedup_enabled {
        fields.iter()
            .filter(|f| f.is_dedup_key)
            .map(|f| f.id)
            .collect()
    } else {
        vec![]
    };

    for (file_idx, file_path) in file_paths.iter().enumerate() {
        // 检查取消状态
        if control.cancelled.load(Ordering::SeqCst) {
            update_task_status(&db, task_id, "cancelled".to_string()).await?;
            return Ok(());
        }

        // 等待恢复
        while control.paused.load(Ordering::SeqCst) {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if control.cancelled.load(Ordering::SeqCst) {
                update_task_status(&db, task_id, "cancelled".to_string()).await?;
                return Ok(());
            }
        }

        let file_name = std::path::Path::new(file_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // 发送文件开始事件
        ProcessingEvent {
            event: "file_start".to_string(),
            task_id: task_id.to_string(),
            current_file: Some(file_name.clone()),
            message: Some(format!("开始处理文件: {}", file_name)),
            ..Default::default()
        }.emit(&app);

        // 处理文件
        let result = process_single_file(
            &app,
            &db,
            task_id,
            file_path.clone(),
            &file_name,
            fields,
            api_url,
            api_key,
            model_name,
            temperature,
            max_tokens,
            &dedup_fields,
            project.dedup_enabled,
            &control,
        ).await;

        match result {
            Ok((rows, success, errors)) => {
                total_rows += rows;
                processed_rows += rows;
                success_count += success;
                error_count += errors;
            }
            Err(e) => {
                error_count += 1;
                ProcessingEvent {
                    event: "error".to_string(),
                    task_id: task_id.to_string(),
                    current_file: Some(file_name.clone()),
                    message: Some(e),
                    ..Default::default()
                }.emit(&app);
            }
        }

        // 更新任务进度
        update_task_progress(&db, task_id, (file_idx + 1) as i32, total_rows, processed_rows, success_count, error_count).await?;

        // 发送文件完成事件
        ProcessingEvent {
            event: "file_complete".to_string(),
            task_id: task_id.to_string(),
            current_file: Some(file_name.clone()),
            processed_rows: Some(processed_rows),
            success_count: Some(success_count),
            error_count: Some(error_count),
            message: Some(format!("文件处理完成: {} 行", processed_rows)),
            ..Default::default()
        }.emit(&app);
    }

    // 更新任务为完成
    update_task_status(&db, task_id, "completed".to_string()).await?;

    // 发送完成事件
    ProcessingEvent {
        event: "completed".to_string(),
        task_id: task_id.to_string(),
        processed_rows: Some(processed_rows),
        success_count: Some(success_count),
        error_count: Some(error_count),
        message: Some(format!("处理完成: 成功 {} 行, 失败 {} 行", success_count, error_count)),
        ..Default::default()
    }.emit(&app);

    Ok(())
}

async fn process_single_file(
    app: &AppHandle,
    db: &Arc<DatabaseConnection>,
    task_id: &str,
    file_path: String,
    file_name: &str,
    fields: &[FieldModel],
    api_url: &str,
    api_key: &str,
    model_name: &str,
    temperature: f32,
    max_tokens: i32,
    dedup_fields: &[i32],
    dedup_enabled: bool,
    control: &Arc<TaskControl>,
) -> Result<(i32, i32, i32), String> {
    let mut total_rows = 0i32;
    let mut success_count = 0i32;
    let error_count = 0i32;

    // 使用 spawn_blocking 读取 Excel 并处理所有 sheets
    let result = tokio::task::spawn_blocking(move || {
        let mut workbook = open_workbook_auto(file_path)
            .map_err(|e| format!("无法打开文件: {}", e))?;
        let sheet_names = workbook.sheet_names().to_vec();
        let mut all_rows: HashMap<String, Vec<Vec<String>>> = HashMap::new();

        for sheet_name in &sheet_names {
            let range = workbook.worksheet_range(sheet_name)
                .map_err(|e| format!("无法读取 Sheet: {}", e))?;
            let rows: Vec<Vec<String>> = range
                .rows()
                .map(|row| row.iter().map(data_to_string).collect())
                .collect();
            all_rows.insert(sheet_name.clone(), rows);
        }

        Ok::<_, String>((sheet_names, all_rows))
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
    .map_err(|e| format!("无法打开文件: {}", e))?;

    let (sheet_names, mut all_rows) = result;

    for sheet_name in sheet_names {
        // 检查取消状态
        if control.cancelled.load(Ordering::SeqCst) {
            return Ok((total_rows, success_count, error_count));
        }

        // 发送 Sheet 开始事件
        ProcessingEvent {
            event: "sheet_start".to_string(),
            task_id: task_id.to_string(),
            current_file: Some(file_name.to_string()),
            current_sheet: Some(sheet_name.clone()),
            message: Some(format!("开始处理 Sheet: {}", sheet_name)),
            ..Default::default()
        }.emit(app);

        // 获取已读取的 Sheet 数据
        let rows_data = match all_rows.remove(&sheet_name) {
            Some(rows) => rows,
            None => continue,
        };

        if rows_data.is_empty() {
            continue;
        }

        // AI 分析列映射
        ProcessingEvent {
            event: "ai_analyzing".to_string(),
            task_id: task_id.to_string(),
            current_sheet: Some(sheet_name.clone()),
            message: Some("AI 分析列映射...".to_string()),
            ..Default::default()
        }.emit(app);

        // 构建字段定义
        let field_defs: Vec<FieldDefinition> = fields.iter().map(|f| FieldDefinition {
            field_name: f.field_name.clone(),
            field_label: f.field_label.clone(),
            field_type: f.field_type.clone(),
            additional_requirement: f.additional_requirement.clone(),
        }).collect();

        // AI 分析
        let mapping_result = analyze_columns_with_ai(
            api_url,
            api_key,
            model_name,
            temperature,
            max_tokens,
            &rows_data[0],
            &field_defs,
            rows_data.get(1..11).map(|r| r.to_vec()),
        ).await?;

        // 发送列映射结果
        let mappings_json: HashMap<String, String> = mapping_result.mappings.iter()
            .map(|m| (m.field_name.clone(), m.column_index.to_string()))
            .collect();

        ProcessingEvent {
            event: "column_mapping".to_string(),
            task_id: task_id.to_string(),
            current_sheet: Some(sheet_name.clone()),
            confidence: Some(mapping_result.confidence),
            mappings: Some(mappings_json.clone()),
            message: Some(format!("列映射完成 (置信度: {:.0}%)", mapping_result.confidence * 100.0)),
            ..Default::default()
        }.emit(app);

        // 创建字段 ID 到索引的映射（预留用于未来优化）
        let _field_id_to_idx: HashMap<i32, usize> = fields.iter()
            .enumerate()
            .map(|(i, f)| (f.id, i))
            .collect();

        // 处理数据行
        let header_row = mapping_result.header_row.max(0) as usize;
        let start_row = header_row + 1;

        let mut empty_count = 0;

        for (row_idx, row) in rows_data.iter().enumerate().skip(start_row) {
            // 检查取消状态
            if control.cancelled.load(Ordering::SeqCst) {
                break;
            }

            // 检查暂停状态
            while control.paused.load(Ordering::SeqCst) {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                if control.cancelled.load(Ordering::SeqCst) {
                    return Ok((total_rows, success_count, error_count));
                }
            }

            // 空行检测
            let is_empty = row.iter().all(|c| c.trim().is_empty());
            if is_empty {
                empty_count += 1;
                if empty_count >= 10 {
                    break; // 连续 10 个空行，跳到下一个 sheet
                }
                continue;
            }
            empty_count = 0;

            total_rows += 1;

            // 提取数据
            let mut data: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            let mut validation_errors = Vec::new();

            for mapping in &mapping_result.mappings {
                if let Some(field) = fields.iter().find(|f| f.field_name == mapping.field_name) {
                    let col_idx = mapping.column_index as usize;
                    if col_idx < row.len() {
                        // 根据字段类型清理数据
                        let value = clean_value(&row[col_idx], &field.field_type);

                        // 验证
                        let rule = field.validation_rule.as_deref();
                        if !validate_value(&value, rule) {
                            validation_errors.push(format!("{} 验证失败", field.field_label));
                        }

                        // 存储（使用 field_id 作为 key）
                        data.insert(field.id.to_string(), serde_json::Value::String(value));
                    }
                }
            }

            // 去重检查
            let is_duplicate = if dedup_enabled && !dedup_fields.is_empty() {
                let mut dedup_values: HashMap<String, String> = HashMap::new();
                for field_id in dedup_fields {
                    if let Some(val) = data.get(&field_id.to_string()) {
                        if let Some(s) = val.as_str() {
                            dedup_values.insert(field_id.to_string(), s.to_string());
                        }
                    }
                }
                check_duplicate(db, task_id, &dedup_values).await?
            } else {
                false
            };

            // 插入记录
            let _status = if validation_errors.is_empty() && !is_duplicate {
                let data_json = serde_json::Value::Object(data);
                insert_record(
                    db,
                    task_id,
                    &data_json,
                    Some(file_name.to_string()),
                    Some(sheet_name.clone()),
                    Some(row_idx as i32),
                ).await?;
                success_count += 1;
                "success".to_string()
            } else if is_duplicate {
                "duplicate".to_string()
            } else {
                "validation_error".to_string()
            };

            // 每 10 行发送进度事件
            if total_rows % 10 == 0 {
                ProcessingEvent {
                    event: "row_processed".to_string(),
                    task_id: task_id.to_string(),
                    current_row: Some(row_idx as i32),
                    total_rows: Some(total_rows),
                    processed_rows: Some(total_rows),
                    success_count: Some(success_count),
                    error_count: Some(error_count),
                    message: Some(format!("已处理 {} 行", total_rows)),
                    ..Default::default()
                }.emit(app);
            }
        }

        // Sheet 完成
        ProcessingEvent {
            event: "sheet_complete".to_string(),
            task_id: task_id.to_string(),
            current_sheet: Some(sheet_name.clone()),
            message: Some(format!("Sheet {} 处理完成", sheet_name)),
            ..Default::default()
        }.emit(app);
    }

    Ok((total_rows, success_count, error_count))
}

async fn analyze_columns_with_ai(
    api_url: &str,
    api_key: &str,
    model_name: &str,
    temperature: f32,
    max_tokens: i32,
    headers: &[String],
    field_defs: &[FieldDefinition],
    sample_rows: Option<Vec<Vec<String>>>,
) -> Result<super::ai_service::ColumnMappingResponse, String> {
    let system_prompt = r#"你是一个数据处理专家，负责分析 Excel 表格的列与目标字段的映射关系。

任务：
1. 识别表头所在行（通常是第一行包含字段名的行）
2. 分析每一列与目标字段的匹配关系
3. 返回 JSON 格式的映射结果

返回格式（必须严格遵循）：
{
  "header_row": 0,
  "mappings": [
    {"field_name": "目标字段名", "column_index": 0, "column_header": "Excel表头", "confidence": 0.95}
  ],
  "confidence": 0.9,
  "unmatched_columns": []
}

注意：
- header_row 从 0 开始计数，-1 表示没有表头
- column_index 从 0 开始
- confidence 范围 0-1，表示匹配置信度
- 如果某列无法匹配任何目标字段，放入 unmatched_columns"#;

    let mut user_prompt = String::new();
    user_prompt.push_str("Excel 表头（按顺序）：\n");
    for (i, header) in headers.iter().enumerate() {
        user_prompt.push_str(&format!("  [{}] {}\n", i, header));
    }

    user_prompt.push_str("\n目标字段定义：\n");
    for field in field_defs {
        let extra = field.additional_requirement
            .as_ref()
            .map(|r| format!(" ({})", r))
            .unwrap_or_default();
        user_prompt.push_str(&format!(
            "  - {} [{}]{}: {}\n",
            field.field_name, field.field_type, extra, field.field_label
        ));
    }

    if let Some(rows) = sample_rows {
        user_prompt.push_str("\n样本数据（前几行）：\n");
        for (i, row) in rows.iter().enumerate() {
            user_prompt.push_str(&format!("  行 {}: {}\n", i, row.join(" | ")));
        }
    }

    user_prompt.push_str("\n请分析列映射关系并返回 JSON 结果。");

    let response = call_ai(
        api_url,
        api_key,
        model_name,
        system_prompt,
        &user_prompt,
        temperature,
        max_tokens,
    ).await?;

    // 解析响应
    let json_str = extract_json(&response)?;
    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;

    let header_row = parsed["header_row"].as_i64().unwrap_or(0) as i32;
    let confidence = parsed["confidence"].as_f64().unwrap_or(0.8) as f32;

    let mappings: Vec<super::ai_service::FieldMapping> = parsed["mappings"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    Some(super::ai_service::FieldMapping {
                        field_name: m["field_name"].as_str()?.to_string(),
                        column_index: m["column_index"].as_i64()? as i32,
                        column_header: m["column_header"].as_str().unwrap_or("").to_string(),
                        confidence: m["confidence"].as_f64().unwrap_or(0.8) as f32,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let unmatched_columns: Vec<i32> = parsed["unmatched_columns"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().map(|i| i as i32))
                .collect()
        })
        .unwrap_or_default();

    Ok(super::ai_service::ColumnMappingResponse {
        header_row,
        mappings,
        confidence,
        unmatched_columns,
    })
}

async fn check_duplicate(db: &Arc<DatabaseConnection>, task_id: &str, dedup_values: &HashMap<String, String>) -> Result<bool, String> {
    // 从 task_id 获取 project_id
    let task = ProcessingTask::find_by_id(task_id)
        .one(db.as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    if let Some(task) = task {
        let mut conditions = vec!["project_id = ?".to_string()];
        let mut params: Vec<sea_orm::Value> = vec![task.project_id.into()];

        for (field_id, value) in dedup_values {
            if !value.trim().is_empty() {
                conditions.push(format!("json_extract(data, '$.{}') = ?", field_id));
                params.push(value.clone().into());
            }
        }

        if conditions.len() > 1 {
            let sql = format!(
                "SELECT id FROM project_records WHERE {} LIMIT 1",
                conditions.join(" AND ")
            );

            let result = db.as_ref()
                .query_one(Statement::from_sql_and_values(
                    db.as_ref().get_database_backend(),
                    &sql,
                    params,
                ))
                .await
                .map_err(|e| format!("数据库错误: {}", e))?;

            return Ok(result.is_some());
        }
    }

    Ok(false)
}

async fn insert_record(
    db: &Arc<DatabaseConnection>,
    task_id: &str,
    data: &serde_json::Value,
    source_file: Option<String>,
    source_sheet: Option<String>,
    row_number: Option<i32>,
) -> Result<i32, String> {
    let task = ProcessingTask::find_by_id(task_id)
        .one(db.as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("任务 {} 不存在", task_id))?;

    let now = chrono::Utc::now().to_rfc3339();
    let data_str = serde_json::to_string(data)
        .map_err(|e| format!("JSON 序列化错误: {}", e))?;

    let new_record = record::ActiveModel {
        project_id: Set(task.project_id),
        data: Set(data_str),
        source_file: Set(source_file),
        source_sheet: Set(source_sheet),
        row_number: Set(row_number),
        batch_number: Set(task.batch_number.clone()),
        status: Set("success".to_string()),
        error_message: Set(None),
        created_at: Set(now),
        updated_at: Set(None),
        ..Default::default()
    };

    let result = new_record
        .insert(db.as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.id)
}

async fn update_task_status(db: &Arc<DatabaseConnection>, task_id: &str, status: String) -> Result<(), String> {
    let task = ProcessingTask::find_by_id(task_id)
        .one(db.as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("任务 {} 不存在", task_id))?;

    let mut active: task::ActiveModel = task.into();
    active.status = Set(status);
    active.updated_at = Set(Some(chrono::Utc::now()));

    active.update(db.as_ref()).await.map_err(|e| format!("数据库错误: {}", e))?;

    Ok(())
}

async fn update_task_progress(
    db: &Arc<DatabaseConnection>,
    task_id: &str,
    processed_files: i32,
    total_rows: i32,
    processed_rows: i32,
    success_count: i32,
    error_count: i32,
) -> Result<(), String> {
    let task = ProcessingTask::find_by_id(task_id)
        .one(db.as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("任务 {} 不存在", task_id))?;

    let mut active: task::ActiveModel = task.into();
    active.processed_files = Set(processed_files);
    active.total_rows = Set(total_rows);
    active.processed_rows = Set(processed_rows);
    active.success_count = Set(success_count);
    active.error_count = Set(error_count);
    active.updated_at = Set(Some(chrono::Utc::now()));

    active.update(db.as_ref()).await.map_err(|e| format!("数据库错误: {}", e))?;

    Ok(())
}

/// 暂停任务
#[tauri::command]
pub async fn pause_processing_task(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    task_id: String,
) -> Result<(), String> {
    let tasks = ACTIVE_TASKS.read().await;
    if let Some(control) = tasks.get(&task_id) {
        control.paused.store(true, Ordering::SeqCst);
    }
    update_task_status(&db, &task_id, "paused".to_string()).await
}

/// 恢复任务
#[tauri::command]
pub async fn resume_processing_task(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    task_id: String,
) -> Result<(), String> {
    let tasks = ACTIVE_TASKS.read().await;
    if let Some(control) = tasks.get(&task_id) {
        control.paused.store(false, Ordering::SeqCst);
    }
    update_task_status(&db, &task_id, "processing".to_string()).await
}

/// 取消任务
#[tauri::command]
pub async fn cancel_processing_task(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    task_id: String,
) -> Result<(), String> {
    let tasks = ACTIVE_TASKS.read().await;
    if let Some(control) = tasks.get(&task_id) {
        control.cancelled.store(true, Ordering::SeqCst);
    }
    update_task_status(&db, &task_id, "cancelled".to_string()).await
}
