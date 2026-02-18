// AI 服务 Tauri Commands
//
// 实现 AI 调用功能，包括列映射分析等

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::backend::infrastructure::{
    config::decrypt,
    persistence::models::AiConfig,
};
use super::ai_utils::{call_ai, extract_json};

// ============ 请求/响应结构 ============

/// 列映射分析请求
#[derive(Debug, Deserialize)]
pub struct AnalyzeColumnMappingRequest {
    pub ai_config_id: i32,
    pub sheet_headers: Vec<String>,        // Excel 表头列表
    pub field_definitions: Vec<FieldDefinition>, // 项目字段定义
    pub sample_rows: Option<Vec<Vec<String>>>,   // 样本数据（前几行）
}

/// 字段定义
#[derive(Debug, Deserialize, Clone)]
pub struct FieldDefinition {
    pub field_name: String,
    pub field_label: String,
    pub field_type: String,
    pub additional_requirement: Option<String>,
}

/// 列映射分析结果
#[derive(Debug, Serialize)]
pub struct ColumnMappingResponse {
    pub header_row: i32,  // 表头所在行（-1 表示无表头）
    pub mappings: Vec<FieldMapping>,
    pub confidence: f32,  // 整体置信度
    pub unmatched_columns: Vec<i32>,  // 未匹配的列索引
}

/// 字段映射
#[derive(Debug, Serialize)]
pub struct FieldMapping {
    pub field_name: String,
    pub column_index: i32,
    pub column_header: String,
    pub confidence: f32,
}

// ============ Tauri Commands ============

/// 分析列映射
#[tauri::command]
pub async fn analyze_column_mapping(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    ai_config_id: i32,
    sheet_headers: Vec<String>,
    field_definitions: Vec<FieldDefinition>,
    sample_rows: Option<Vec<Vec<String>>>,
) -> Result<ColumnMappingResponse, String> {
    // 获取 AI 配置
    let config = AiConfig::find_by_id(ai_config_id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("AI 配置 {} 不存在", ai_config_id))?;

    // 解密 API Key
    let api_key = decrypt(&config.api_key)
        .map_err(|e| format!("解密失败: {}", e))?;

    // 构建 AI 请求
    let system_prompt = build_system_prompt();
    let user_prompt = build_user_prompt(&sheet_headers, &field_definitions, &sample_rows);

    // 调用 AI
    let response = call_ai(
        &config.api_url,
        &api_key,
        &config.model_name,
        &system_prompt,
        &user_prompt,
        config.temperature,
        config.max_tokens,
    ).await?;

    // 解析 AI 响应
    parse_mapping_response(&response)
}

/// AI 辅助生成字段元数据（仅翻译字段名）
#[tauri::command]
pub async fn ai_generate_field_metadata(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    ai_config_id: i32,
    field_label: String,
    field_type: String,
    _additional_requirement: Option<String>,
) -> Result<serde_json::Value, String> {
    // 获取 AI 配置
    let config = AiConfig::find_by_id(ai_config_id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("AI 配置 {} 不存在", ai_config_id))?;

    // 解密 API Key
    let api_key = decrypt(&config.api_key)
        .map_err(|e| format!("解密失败: {}", e))?;

    let system_prompt = r#"你是一个字段名翻译专家。将中文字段标签翻译成英文变量名。

规则：
- 返回小写英文字段名，使用下划线分隔单词
- 简洁、常见、易于理解
- 只返回字段名本身，不要有任何其他内容

示例：
"姓名" -> "name"
"手机号码" -> "phone"
"电子邮箱" -> "email"
"公司名称" -> "company_name"
"收货地址" -> "shipping_address"
"创建时间" -> "created_at""#;

    let user_prompt = format!("翻译：{}", field_label);

    let response = call_ai(
        &config.api_url,
        &api_key,
        &config.model_name,
        system_prompt,
        &user_prompt,
        0.1,  // 极低温度确保稳定输出
        50,   // 只需要几个词
    ).await?;

    // 清理响应（去除可能的引号、空格、换行）
    let field_name = response
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_lowercase()
        .replace(' ', "_")
        .replace('-', "_");

    // 验证字段名有效性
    let field_name = if field_name.is_empty() || !field_name.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
        field_label.to_lowercase().replace(' ', "_")
    } else {
        field_name
    };

    // 构建返回结果（验证规则使用内置逻辑）
    Ok(serde_json::json!({
        "field_name": field_name,
        "validation_rule": get_builtin_validation_rule(&field_type),
        "extraction_hint": format!("提取{}字段", field_label)
    }))
}

/// 根据字段类型获取内置验证规则
fn get_builtin_validation_rule(field_type: &str) -> Option<String> {
    match field_type {
        "phone" => Some(r"^1[3-9]\d{9}$".to_string()),
        "email" => Some(r"^[\w\.-]+@[\w\.-]+\.\w+$".to_string()),
        "url" => Some(r"^https?://".to_string()),
        "date" => Some(r"^\d{4}[-/]\d{1,2}[-/]\d{1,2}$".to_string()),
        "number" => Some(r"^-?\d+(\.\d+)?$".to_string()),
        _ => None, // text 等类型无需验证
    }
}

// ============ 辅助函数 ============

/// 构建系统提示
fn build_system_prompt() -> &'static str {
    r#"你是一个数据处理专家，负责分析 Excel 表格的列与目标字段的映射关系。

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
- 如果某列无法匹配任何目标字段，放入 unmatched_columns"#
}

/// 构建用户提示
fn build_user_prompt(
    sheet_headers: &[String],
    field_definitions: &[FieldDefinition],
    sample_rows: &Option<Vec<Vec<String>>>,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("Excel 表头（按顺序）：\n");
    for (i, header) in sheet_headers.iter().enumerate() {
        prompt.push_str(&format!("  [{}] {}\n", i, header));
    }

    prompt.push_str("\n目标字段定义：\n");
    for field in field_definitions {
        let extra = field.additional_requirement
            .as_ref()
            .map(|r| format!(" ({})", r))
            .unwrap_or_default();
        prompt.push_str(&format!(
            "  - {} [{}]{}: {}\n",
            field.field_name, field.field_type, extra, field.field_label
        ));
    }

    if let Some(rows) = sample_rows {
        prompt.push_str("\n样本数据（前几行）：\n");
        for (i, row) in rows.iter().enumerate() {
            prompt.push_str(&format!("  行 {}: {}\n", i, row.join(" | ")));
        }
    }

    prompt.push_str("\n请分析列映射关系并返回 JSON 结果。");
    prompt
}

/// 解析映射响应
fn parse_mapping_response(response: &str) -> Result<ColumnMappingResponse, String> {
    // 尝试提取 JSON（AI 可能会在前后加一些说明文字）
    let json_str = extract_json(response)?;

    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;

    let header_row = parsed["header_row"].as_i64().unwrap_or(0) as i32;
    let confidence = parsed["confidence"].as_f64().unwrap_or(0.8) as f32;

    let mappings = parsed["mappings"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    Some(FieldMapping {
                        field_name: m["field_name"].as_str()?.to_string(),
                        column_index: m["column_index"].as_i64()? as i32,
                        column_header: m["column_header"].as_str().unwrap_or("").to_string(),
                        confidence: m["confidence"].as_f64().unwrap_or(0.8) as f32,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let unmatched_columns = parsed["unmatched_columns"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().map(|i| i as i32))
                .collect()
        })
        .unwrap_or_default();

    Ok(ColumnMappingResponse {
        header_row,
        mappings,
        confidence,
        unmatched_columns,
    })
}
