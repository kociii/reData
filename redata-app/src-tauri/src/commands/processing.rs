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

use super::tasks::upsert_file_progress;

/// 将一行数据格式化为索引字符串，格式：1:列1内容;2:列2内容;...n:列n内容;
fn format_row_indexed(row: &[String]) -> String {
    row.iter()
        .enumerate()
        .map(|(i, val)| format!("{}:{};", i + 1, val))
        .collect()
}

/// 根据字段类型获取识别规则
fn get_field_type_rules(field_type: &str) -> &'static str {
    match field_type {
        "company" => "数据应含\"有限公司\"、\"有限责任公司\"、\"股份公司\"、\"集团\"、Inc、Ltd、Corp、Co.、LLC等企业实体标识；列名含\"客户\"、\"卖家\"但数据为纯数字/纯字母编号时，是ID列而非公司名，不得映射。\n    ⚠️ 严禁映射：纯数字列（如ID、编号、订单号等）绝不能映射为公司名称，即使列名含有\"客户\"或\"卖家\"等词语",
        "phone" => "数据应为11位手机号（1开头）或固话格式（区号-号码），纯数字但不符合手机/固话格式的不得映射",
        "email" => "数据必须包含@符号，格式为 xxx@xxx.xxx",
        "name" => "数据通常为2-4个中文字符或英文人名；若数据含\"公司\"、\"有限\"、\"集团\"等词则为企业名，不得映射为姓名",
        "address" => "数据应包含省/市/区/路/街/号/楼等地址成分；单纯的城市名或省份名不是完整地址",
        "date" => "数据应为日期格式如 YYYY-MM-DD、YYYY/MM/DD、MM/DD/YYYY 等；纯数字时间戳不是日期",
        "number" => "数据应为纯数字、整数或小数；含字母或特殊符号的编号不是数字字段",
        "id_card" => "数据应为15位纯数字或18位（前17位数字+最后1位数字或X）的身份证号格式",
        "url" => "数据必须以 http:// 或 https:// 开头",
        "text" => "通用文本字段，列名语义匹配即可，但不应映射已被其他类型明确拒绝的列",
        _ => "根据列名语义和样本数据内容综合判断"
    }
}
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
use super::ai_utils::{call_ai_stream, extract_json};
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
    /// Sheet 级别的成功计数（sheet_complete 事件）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sheet_success_count: Option<i32>,
    /// Sheet 级别的错误计数（sheet_complete 事件）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sheet_error_count: Option<i32>,
    /// Sheet 级别的总行数（sheet_complete 事件）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sheet_total_rows: Option<i32>,
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
    pub source_files: Vec<String>,
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
        "company" => {
            // 公司名称：压缩空白；若清理后为纯数字（如 ID、编号），视为无效值返回空
            let mut result = String::new();
            let mut prev_space = false;
            for c in cleaned.chars() {
                if c.is_whitespace() {
                    if !prev_space { result.push(' '); prev_space = true; }
                } else {
                    result.push(c);
                    prev_space = false;
                }
            }
            cleaned = result.trim().to_string();
            // 纯数字（含空格分隔）不是公司名称，清空
            if !cleaned.is_empty() && cleaned.chars().all(|c| c.is_ascii_digit() || c.is_whitespace()) {
                cleaned = String::new();
            }
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
    // 提取源文件名列表
    let source_file_names: Vec<String> = file_paths
        .iter()
        .map(|p| {
            std::path::Path::new(p)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        })
        .collect();
    let source_files_json = serde_json::to_string(&source_file_names)
        .unwrap_or_else(|_| "[]".to_string());

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
        source_files: Set(Some(source_files_json)),
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
        source_files: source_file_names,
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

        // 持久化：创建文件进度记录
        let _ = upsert_file_progress(
            &db,
            task_id,
            &file_name,
            None,  // sheet_name 为空表示文件级别
            Some("processing"),
            None,  // sheet_phase
            None,  // ai_confidence
            None,  // mapping_count
            None,  // success_count
            None,  // error_count
            None,  // total_rows
            None,  // error_message
        ).await;

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

                // 持久化：更新文件完成状态
                let _ = upsert_file_progress(
                    &db,
                    task_id,
                    &file_name,
                    None,
                    Some("done"),
                    None,  // sheet_phase
                    None,  // ai_confidence
                    None,  // mapping_count
                    Some(success),
                    Some(errors),
                    Some(rows),
                    None,  // error_message
                ).await;
            }
            Err(e) => {
                error_count += 1;

                // 持久化：更新文件错误状态
                let _ = upsert_file_progress(
                    &db,
                    task_id,
                    &file_name,
                    None,
                    Some("error"),
                    None,  // sheet_phase
                    None,  // ai_confidence
                    None,  // mapping_count
                    None,  // success_count
                    None,  // error_count
                    None,  // total_rows
                    Some(&e),
                ).await;

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

        // 记录 Sheet 开始时的基线值（用于计算当前 Sheet 的增量）
        let sheet_start_total = total_rows;
        let sheet_start_success = success_count;
        let sheet_start_error = error_count;

        // 发送 Sheet 开始事件
        ProcessingEvent {
            event: "sheet_start".to_string(),
            task_id: task_id.to_string(),
            current_file: Some(file_name.to_string()),
            current_sheet: Some(sheet_name.clone()),
            message: Some(format!("开始处理 Sheet: {}", sheet_name)),
            ..Default::default()
        }.emit(app);

        // 持久化：创建 Sheet 进度记录
        let _ = upsert_file_progress(
            db,
            task_id,
            file_name,
            Some(&sheet_name),
            None,  // file_phase 不变
            Some("ai_analyzing"),
            None,  // ai_confidence
            None,  // mapping_count
            None,  // success_count
            None,  // error_count
            None,  // total_rows
            None,  // error_message
        ).await;

        // 获取已读取的 Sheet 数据
        let rows_data = match all_rows.remove(&sheet_name) {
            Some(rows) => rows,
            None => {
                // Sheet 数据不存在，标记为完成（0 行）
                let _ = upsert_file_progress(
                    db, task_id, file_name, Some(&sheet_name),
                    None, Some("done"), None, None,
                    Some(0), Some(0), Some(0), None,
                ).await;
                ProcessingEvent {
                    event: "sheet_complete".to_string(),
                    task_id: task_id.to_string(),
                    current_file: Some(file_name.to_string()),
                    current_sheet: Some(sheet_name.clone()),
                    sheet_success_count: Some(0),
                    sheet_error_count: Some(0),
                    sheet_total_rows: Some(0),
                    message: Some(format!("Sheet {} 无数据", sheet_name)),
                    ..Default::default()
                }.emit(app);
                continue;
            }
        };

        if rows_data.is_empty() {
            // Sheet 为空，标记为完成（0 行）
            let _ = upsert_file_progress(
                db, task_id, file_name, Some(&sheet_name),
                None, Some("done"), None, None,
                Some(0), Some(0), Some(0), None,
            ).await;
            ProcessingEvent {
                event: "sheet_complete".to_string(),
                task_id: task_id.to_string(),
                current_file: Some(file_name.to_string()),
                current_sheet: Some(sheet_name.clone()),
                sheet_success_count: Some(0),
                sheet_error_count: Some(0),
                sheet_total_rows: Some(0),
                message: Some(format!("Sheet {} 无数据，跳过", sheet_name)),
                ..Default::default()
            }.emit(app);
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
            extraction_hint: f.extraction_hint.clone(),
        }).collect();

        // AI 分析（流式）
        let app_clone = app.clone();
        let task_id_clone = task_id.to_string();
        let sheet_name_clone = sheet_name.clone();

        // 构建请求提示（用于显示）- 只取前 5 行样本数据
        let request_preview = build_request_preview(&rows_data[0], &field_defs, rows_data.get(1..6).map(|r| r.to_vec()));
        ProcessingEvent {
            event: "ai_request".to_string(),
            task_id: task_id.to_string(),
            current_sheet: Some(sheet_name.clone()),
            message: Some(request_preview),
            ..Default::default()
        }.emit(app);

        let mapping_result = analyze_columns_with_ai_stream(
            app_clone,
            api_url,
            api_key,
            model_name,
            temperature,
            max_tokens,
            &rows_data[0],
            &field_defs,
            rows_data.get(1..6).map(|r| r.to_vec()),  // 只取前 5 行样本数据
            task_id_clone,
            sheet_name_clone,
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

        // 持久化：更新 AI 置信度和映射数
        let _ = upsert_file_progress(
            db,
            task_id,
            file_name,
            Some(&sheet_name),
            None,  // file_phase
            Some("importing"),
            Some(mapping_result.confidence),
            Some(mapping_result.mappings.len() as i32),
            None,  // success_count
            None,  // error_count
            None,  // total_rows
            None,  // error_message
        ).await;

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

                        // 必填字段验证
                        if field.is_required && value.trim().is_empty() {
                            validation_errors.push(format!("{} 为必填项", field.field_label));
                        }

                        // 格式验证
                        let rule = field.validation_rule.as_deref();
                        if !validate_value(&value, rule) {
                            validation_errors.push(format!("{} 验证失败", field.field_label));
                        }

                        // 存储（使用 field_id 作为 key）
                        data.insert(field.id.to_string(), serde_json::Value::String(value));
                    } else if field.is_required {
                        // 列不存在但字段必填
                        validation_errors.push(format!("{} 为必填项", field.field_label));
                    }
                }
            }

            // 检查必填字段是否在 AI 映射中完全缺失（AI 未能找到对应列）
            let mapped_field_names: std::collections::HashSet<&str> = mapping_result.mappings
                .iter()
                .map(|m| m.field_name.as_str())
                .collect();
            for field in fields.iter().filter(|f| f.is_required) {
                if !mapped_field_names.contains(field.field_name.as_str()) {
                    validation_errors.push(format!("{} 为必填项（未找到对应列）", field.field_label));
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
                    Some(row),  // 传递原始行数据
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

        // Sheet 完成时计算当前 Sheet 的增量值
        let sheet_success = success_count - sheet_start_success;  // 当前 Sheet 的成功数（增量）
        let sheet_error = error_count - sheet_start_error;        // 当前 Sheet 的错误数（增量）
        let sheet_total = total_rows - sheet_start_total;         // 当前 Sheet 的总行数（增量）

        // 持久化：更新 Sheet 完成状态和统计
        let _ = upsert_file_progress(
            db,
            task_id,
            file_name,
            Some(&sheet_name),
            None,  // file_phase
            Some("done"),
            None,  // ai_confidence
            None,  // mapping_count
            Some(sheet_success),
            Some(sheet_error),
            Some(sheet_total),
            None,  // error_message
        ).await;

        // Sheet 完成 - 添加 sheet 级别统计字段
        ProcessingEvent {
            event: "sheet_complete".to_string(),
            task_id: task_id.to_string(),
            current_file: Some(file_name.to_string()),
            current_sheet: Some(sheet_name.clone()),
            sheet_success_count: Some(sheet_success),
            sheet_error_count: Some(sheet_error),
            sheet_total_rows: Some(sheet_total),
            message: Some(format!("Sheet {} 处理完成: 成功 {} 行, 失败 {} 行", sheet_name, sheet_success, sheet_error)),
            ..Default::default()
        }.emit(app);
    }

    Ok((total_rows, success_count, error_count))
}

fn build_request_preview(
    headers: &[String],
    field_defs: &[FieldDefinition],
    sample_rows: Option<Vec<Vec<String>>>,
) -> String {
    let mut preview = String::new();
    preview.push_str("📤 发送给 AI 的数据:\n\n");
    preview.push_str("📋 Excel 表头:\n");
    for (i, header) in headers.iter().enumerate() {
        preview.push_str(&format!("  [{}] {}\n", i, header));
    }

    preview.push_str("\n📝 目标字段:\n");
    for field in field_defs {
        let extra = field.additional_requirement
            .as_ref()
            .map(|r| format!(" ({})", r))
            .unwrap_or_default();
        preview.push_str(&format!(
            "  • {} [{}]{}: {}\n",
            field.field_name, field.field_type, extra, field.field_label
        ));
    }

    if let Some(rows) = sample_rows {
        preview.push_str("\n📊 样本数据（列编号从1开始）:\n");
        for (i, row) in rows.iter().enumerate().take(3) {
            preview.push_str(&format!("  行 {}: {}\n", i, format_row_indexed(row)));
        }
    }

    preview
}

async fn analyze_columns_with_ai_stream(
    app: AppHandle,
    api_url: &str,
    api_key: &str,
    model_name: &str,
    temperature: f32,
    max_tokens: i32,
    headers: &[String],
    field_defs: &[FieldDefinition],
    sample_rows: Option<Vec<Vec<String>>>,
    task_id: String,
    sheet_name: String,
) -> Result<super::ai_service::ColumnMappingResponse, String> {
    let system_prompt = r#"你是专业的 Excel 数据结构分析专家，负责将 Excel 列精准映射到目标字段。

## 核心原则：两步验证（缺一不可）

### 第一步：列名语义匹配
表头/列名在语义上是否对应目标字段。

### 第二步：数据内容验证（最重要）
逐列检查样本数据，验证实际内容是否符合字段类型的数据特征：

| 字段类型 | 数据内容必须满足 | 常见误判陷阱 |
|---------|----------------|------------|
| company | 含"有限公司"、"集团"、Inc、Ltd、Corp 等文字 | ❌ 纯数字/纯字母编号列名含"客户"→ID列，不是公司名 |
| phone   | 11位手机号或固话格式 | ❌ 含字母的编号不是电话 |
| email   | 包含 @ 符号 | ❌ 没有@的字符串不是邮箱 |
| name    | 2-4个中文字符或英文人名 | ❌ 含"公司"/"集团"的是企业名不是姓名 |
| address | 含省/市/区/路/号/街道等 | ❌ 纯城市名不是完整地址 |
| date    | YYYY-MM-DD 等日期格式 | ❌ 纯数字时间戳不是日期 |
| number  | 纯数字或小数 | ❌ 含字母的编号不是数字字段 |
| id_card | 15或18位含字母X的身份证格式 | ❌ 普通15位数字不是身份证 |
| url     | 以 http:// 或 https:// 开头 | ❌ 没有协议前缀不是URL |
| text    | 通用文本，列名语义匹配即可 | — |

## 决策规则
- ✅ 两步均匹配 → 建立映射，confidence 反映确定程度
- ❌ 任意一步不匹配 → 放入 unmatched_columns，**宁缺毋滥**

## 返回格式（严格 JSON）
{
  "header_row": 0,
  "mappings": [
    {"field_name": "字段名", "column_index": 0, "column_header": "Excel列名", "confidence": 0.95}
  ],
  "confidence": 0.9,
  "unmatched_columns": [1, 3]
}

header_row 和 column_index 均从 0 计数；-1 表示无表头"#;

    // 列维度展示：表头 + 该列的样本值（方便 AI 逐列验证数据内容）
    let mut user_prompt = String::new();
    user_prompt.push_str("## Excel 列数据预览（列名 → 样本值）\n\n");
    for (col_idx, header) in headers.iter().enumerate() {
        let col_samples: Vec<&str> = sample_rows
            .as_ref()
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| row.get(col_idx).map(|s| s.as_str()))
                    .filter(|s| !s.trim().is_empty())
                    .take(5)
                    .collect()
            })
            .unwrap_or_default();
        if col_samples.is_empty() {
            user_prompt.push_str(&format!("列[{}] \"{}\"  →  (空列)\n", col_idx, header));
        } else {
            user_prompt.push_str(&format!("列[{}] \"{}\"  →  {}\n", col_idx, header, col_samples.join(" | ")));
        }
    }

    // 目标字段定义
    user_prompt.push_str("\n## 目标字段定义\n\n");
    for field in field_defs {
        let type_rules = get_field_type_rules(&field.field_type);
        let extra = field.additional_requirement
            .as_ref()
            .map(|r| format!("（{}）", r))
            .unwrap_or_default();
        let extraction = field.extraction_hint
            .as_ref()
            .map(|h| format!("\n  提取要求: {}", h))
            .unwrap_or_default();
        user_prompt.push_str(&format!(
            "- {} [{}]{}: {}\n  数据特征: {}{}\n",
            field.field_name, field.field_type, extra, field.field_label, type_rules, extraction
        ));
    }

    user_prompt.push_str("\n## 任务\n对每一列执行两步验证（列名语义 + 数据内容），输出 JSON 映射结果。");


    // 使用流式调用，每个 chunk 发送事件
    let app_for_stream = app.clone();
    let task_id_for_stream = task_id.clone();
    let sheet_name_for_stream = sheet_name.clone();

    let response = call_ai_stream(
        api_url,
        api_key,
        model_name,
        system_prompt,
        &user_prompt,
        temperature,
        max_tokens,
        true,  // json_mode: 列映射需要返回 JSON
        move |chunk: &str| {
            // 发送流式事件
            let event = ProcessingEvent {
                event: "ai_response".to_string(),
                task_id: task_id_for_stream.clone(),
                current_sheet: Some(sheet_name_for_stream.clone()),
                message: Some(chunk.to_string()),
                ..Default::default()
            };
            event.emit(&app_for_stream);
        },
    ).await?;

    // 解析响应
    let json_str = extract_json(&response)?;
    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;

    let header_row = parsed["header_row"].as_i64().unwrap_or(0) as i32;
    let confidence = parsed["confidence"].as_f64().unwrap_or(0.8) as f32;

    // 写入 AI 调试日志（写到系统临时目录，避免触发 Tauri 文件监听）
    {
        use std::io::Write;
        let log_path = std::env::temp_dir().join("redata_ai_debug.log");
        let mappings_summary = parsed["mappings"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|m| format!(
                        "  {} -> col[{}] \"{}\" ({:.0}%)",
                        m["field_name"].as_str().unwrap_or("?"),
                        m["column_index"].as_i64().unwrap_or(-1),
                        m["column_header"].as_str().unwrap_or(""),
                        m["confidence"].as_f64().unwrap_or(0.0) * 100.0
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
        let entry = format!(
            "\n====== AI 列映射日志 [Sheet: {}] ======\n\
            ## 请求\n{}\n\n\
            ## AI 原始响应\n{}\n\n\
            ## 解析结果 (header_row={}, confidence={:.0}%)\n{}\n\
            ==========================================\n",
            sheet_name, user_prompt, response, header_row, confidence * 100.0, mappings_summary
        );
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
            let _ = f.write_all(entry.as_bytes());
        }
    }

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
    raw_data: Option<&[String]>,
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

    // 序列化原始行数据为索引格式：1:列1内容;2:列2内容;...n:列n内容;
    let raw_data_str = raw_data.map(|row| format_row_indexed(row));

    let new_record = record::ActiveModel {
        project_id: Set(task.project_id),
        data: Set(data_str),
        raw_data: Set(raw_data_str),
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
