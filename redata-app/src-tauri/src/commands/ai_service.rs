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
    /// AI 提取要求（用户自定义）
    pub extraction_hint: Option<String>,
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
    let system_prompt = build_system_prompt(&field_definitions);
    let user_prompt = build_user_prompt(&sheet_headers, &field_definitions, &sample_rows);

    // 调用 AI（启用 JSON 模式，确保结构化输出）
    let response = call_ai(
        &config.api_url,
        &api_key,
        &config.model_name,
        &system_prompt,
        &user_prompt,
        config.temperature,
        config.max_tokens,
        true,  // json_mode: 列映射需要返回 JSON
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
        false,  // json_mode: 字段名翻译只需纯文本
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

/// 构建系统提示（根据实际字段类型动态生成规则表）
fn build_system_prompt(field_definitions: &[FieldDefinition]) -> String {
    // 收集实际使用的字段类型（去重）
    let used_types: std::collections::HashSet<&str> = field_definitions
        .iter()
        .map(|f| f.field_type.as_str())
        .collect();

    let mut prompt = String::from(
        "你是专业的 Excel 数据结构分析专家，负责将 Excel 列精准映射到目标字段。\n\n\
         ## 核心原则：两步验证（缺一不可）\n\n\
         ### 第一步：列名语义匹配\n\
         表头/列名在语义上是否对应目标字段。\n\n\
         ### 第二步：数据内容验证（最重要）\n\
         逐列检查样本数据，验证实际内容是否符合字段类型的数据特征：\n\n\
         | 字段类型 | 数据内容必须满足 | 常见误判陷阱 |\n\
         |---------|----------------|------------|\n",
    );

    // 类型规则表（按顺序，仅输出实际用到的类型）
    const TYPE_RULE_TABLE: &[(&str, &str, &str)] = &[
        ("company", "含\"有限公司\"、\"集团\"、Inc、Ltd、Corp 等文字", "❌ 纯数字/纯字母编号列名含\"客户\"→ID列，不是公司名"),
        ("phone",   "11位手机号或固话格式",                            "❌ 含字母的编号不是电话"),
        ("email",   "包含 @ 符号",                                    "❌ 没有@的字符串不是邮箱"),
        ("name",    "2-4个中文字符或英文人名",                          "❌ 含\"公司\"/\"集团\"的是企业名不是姓名"),
        ("address", "含省/市/区/路/号/街道等",                          "❌ 纯城市名不是完整地址"),
        ("date",    "YYYY-MM-DD 等日期格式",                           "❌ 纯数字时间戳不是日期"),
        ("number",  "纯数字或小数",                                    "❌ 含字母的编号不是数字字段"),
        ("id_card", "15或18位含字母X的身份证格式",                       "❌ 普通15位数字不是身份证"),
        ("url",     "以 http:// 或 https:// 开头",                    "❌ 没有协议前缀不是URL"),
        ("text",    "通用文本，列名语义匹配 + 满足字段定义中的识别条件",    "—"),
    ];

    for &(type_name, must_satisfy, trap) in TYPE_RULE_TABLE {
        if used_types.contains(type_name) {
            prompt.push_str(&format!("| {} | {} | {} |\n", type_name, must_satisfy, trap));
        }
    }

    prompt.push_str(
        "\n## 决策规则\n\
         - ✅ 两步均匹配 → 建立映射，confidence 反映确定程度\n\
         - ❌ 任意一步不匹配 → 放入 unmatched_columns，**宁缺毋滥**\n\n\
         ## 返回格式（严格 JSON）\n\
         {\n\
           \"header_row\": 0,\n\
           \"mappings\": [\n\
             {\"field_name\": \"字段名\", \"column_index\": 0, \"column_header\": \"Excel列名\", \"confidence\": 0.95}\n\
           ],\n\
           \"confidence\": 0.9,\n\
           \"unmatched_columns\": [1, 3]\n\
         }\n\n\
         header_row 和 column_index 均从 0 计数；-1 表示无表头",
    );

    prompt
}

/// 构建用户提示
fn build_user_prompt(
    sheet_headers: &[String],
    field_definitions: &[FieldDefinition],
    sample_rows: &Option<Vec<Vec<String>>>,
) -> String {
    let mut prompt = String::new();

    // 列维度展示：表头 + 该列的样本值（方便 AI 逐列验证数据内容）
    prompt.push_str("## Excel 列数据预览（列名 → 样本值）\n\n");
    for (col_idx, header) in sheet_headers.iter().enumerate() {
        let samples: Vec<&str> = if let Some(rows) = sample_rows {
            rows.iter()
                .filter_map(|row| row.get(col_idx).map(|s| s.as_str()))
                .filter(|s| !s.trim().is_empty())
                .take(5)
                .collect()
        } else {
            vec![]
        };
        if samples.is_empty() {
            prompt.push_str(&format!("列[{}] \"{}\"  →  (空列)\n", col_idx, header));
        } else {
            prompt.push_str(&format!("列[{}] \"{}\"  →  {}\n", col_idx, header, samples.join(" | ")));
        }
    }

    // 目标字段定义
    prompt.push_str("\n## 目标字段定义\n\n");
    for field in field_definitions {
        let type_rules = get_field_type_rules(&field.field_type);

        // 将 additional_requirement 整合进数据特征：
        // - text 类型：用户输入的识别条件是主要依据
        // - 其他类型：作为附加约束追加到内置规则后
        let data_feature = match (&field.additional_requirement, field.field_type.as_str()) {
            (Some(req), "text") => format!("通用文本字段，识别条件：{}", req),
            (Some(req), _)      => format!("{}；附加约束：{}", type_rules, req),
            (None, _)           => type_rules.to_string(),
        };

        let extraction = field.extraction_hint
            .as_ref()
            .map(|h| format!("\n  提取要求: {}", h))
            .unwrap_or_default();
        prompt.push_str(&format!(
            "- {} [{}]: {}\n  数据特征: {}{}\n",
            field.field_name, field.field_type, field.field_label, data_feature, extraction
        ));
    }

    prompt.push_str("\n## 任务\n对每一列执行两步验证（列名语义 + 数据内容），输出 JSON 映射结果。");
    prompt
}

/// 根据字段类型获取识别规则
fn get_field_type_rules(field_type: &str) -> &'static str {
    match field_type {
        "company" => "数据应含\"有限公司\"、\"有限责任公司\"、\"股份公司\"、\"集团\"、Inc、Ltd、Corp、Co.、LLC等企业实体标识；列名含\"客户\"、\"卖家\"但数据为纯数字/纯字母编号时，是ID列而非公司名，不得映射",
        "phone"   => "数据应为11位手机号（1开头）或固话格式（区号-号码），纯数字但不符合手机/固话格式的不得映射",
        "email"   => "数据必须包含@符号，格式为 xxx@xxx.xxx",
        "name"    => "数据通常为2-4个中文字符或英文人名；若数据含\"公司\"、\"有限\"、\"集团\"等词则为企业名，不得映射为姓名",
        "address" => "数据应包含省/市/区/路/街/号/楼等地址成分；单纯的城市名或省份名不是完整地址",
        "date"    => "数据应为日期格式如 YYYY-MM-DD、YYYY/MM/DD、MM/DD/YYYY 等；纯数字时间戳不是日期",
        "number"  => "数据应为纯数字、整数或小数；含字母或特殊符号的编号不是数字字段",
        "id_card" => "数据应为15位纯数字或18位（前17位数字+最后1位数字或X）的身份证号格式",
        "url"     => "数据必须以 http:// 或 https:// 开头",
        "text"    => "通用文本字段，列名语义匹配即可，但不应映射已被其他类型明确拒绝的列",
        _         => "根据列名语义和样本数据内容综合判断",
    }
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
