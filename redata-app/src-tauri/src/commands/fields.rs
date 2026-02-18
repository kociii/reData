// 字段管理 Tauri Commands
//
// 实现项目字段的 CRUD 操作、软删除和元数据生成
// 利用 Rust 的强类型系统和 async/await 模式

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    Set, Condition,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::backend::infrastructure::persistence::models::{field, ProjectField};

// ============ 请求/响应结构 ============

/// 字段创建请求
#[derive(Debug, Deserialize)]
pub struct CreateFieldRequest {
    pub project_id: i32,
    pub field_name: String,
    pub field_label: String,
    pub field_type: String,
    #[serde(default)]
    pub is_required: bool,
    #[serde(default)]
    pub is_dedup_key: bool,
    pub additional_requirement: Option<String>,
    pub validation_rule: Option<String>,
    pub extraction_hint: Option<String>,
}

/// 字段更新请求
#[derive(Debug, Deserialize)]
pub struct UpdateFieldRequest {
    pub field_name: Option<String>,
    pub field_label: Option<String>,
    pub field_type: Option<String>,
    pub is_required: Option<bool>,
    pub is_dedup_key: Option<bool>,
    pub additional_requirement: Option<String>,
    pub validation_rule: Option<String>,
    pub extraction_hint: Option<String>,
    pub display_order: Option<i32>,
}

/// 字段响应结构
#[derive(Debug, Serialize, Clone)]
pub struct FieldResponse {
    pub id: i32,
    pub project_id: i32,
    pub field_name: String,
    pub field_label: String,
    pub field_type: String,
    pub is_required: bool,
    pub is_dedup_key: bool,
    pub is_deleted: bool,
    pub additional_requirement: Option<String>,
    pub validation_rule: Option<String>,
    pub extraction_hint: Option<String>,
    pub display_order: i32,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

/// 元数据生成请求
#[derive(Debug, Deserialize)]
pub struct GenerateMetadataRequest {
    pub field_label: String,
    pub field_type: String,
    pub additional_requirement: Option<String>,
}

/// 元数据响应
#[derive(Debug, Serialize)]
pub struct MetadataResponse {
    pub field_name: String,
    pub validation_rule: Option<String>,
    pub extraction_hint: String,
}

// ============ 类型转换 ============

impl From<field::Model> for FieldResponse {
    fn from(model: field::Model) -> Self {
        Self {
            id: model.id,
            project_id: model.project_id,
            field_name: model.field_name,
            field_label: model.field_label,
            field_type: model.field_type,
            is_required: model.is_required,
            is_dedup_key: model.is_dedup_key,
            is_deleted: model.is_deleted,
            additional_requirement: model.additional_requirement,
            validation_rule: model.validation_rule,
            extraction_hint: model.extraction_hint,
            display_order: model.display_order,
            created_at: model.created_at.to_rfc3339(),
            deleted_at: model.deleted_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

// ============ 辅助函数 ============

/// 根据字段类型生成本地验证规则（正则表达式）
fn get_validation_rule(field_type: &str) -> Option<String> {
    match field_type {
        "phone" => Some(r"^1[3-9]\d{9}$".to_string()),
        "email" => Some(r"^[\w\.-]+@[\w\.-]+\.\w+$".to_string()),
        "url" => Some(r"^https?://".to_string()),
        "date" => Some(r"^\d{4}[-/]\d{1,2}[-/]\d{1,2}$".to_string()),
        "number" => Some(r"^-?\d+(\.\d+)?$".to_string()),
        _ => None, // text 类型无需验证规则
    }
}

/// 根据字段标签生成英文字段名（常见中文词汇映射）
fn generate_field_name(label: &str) -> String {
    use std::collections::HashMap;

    // 常见字段名中英文映射
    let mut mappings: HashMap<&str, &str> = HashMap::new();
    mappings.insert("姓名", "name");
    mappings.insert("名字", "name");
    mappings.insert("用户名", "username");
    mappings.insert("电话", "phone");
    mappings.insert("手机", "phone");
    mappings.insert("手机号", "phone");
    mappings.insert("手机号码", "phone");
    mappings.insert("邮箱", "email");
    mappings.insert("电子邮件", "email");
    mappings.insert("地址", "address");
    mappings.insert("公司", "company");
    mappings.insert("公司名称", "company_name");
    mappings.insert("企业", "company");
    mappings.insert("日期", "date");
    mappings.insert("时间", "time");
    mappings.insert("金额", "amount");
    mappings.insert("价格", "price");
    mappings.insert("数量", "quantity");
    mappings.insert("备注", "remark");
    mappings.insert("说明", "description");
    mappings.insert("描述", "description");
    mappings.insert("标题", "title");
    mappings.insert("编号", "id");
    mappings.insert("序号", "serial_number");
    mappings.insert("状态", "status");
    mappings.insert("类型", "type");
    mappings.insert("分类", "category");
    mappings.insert("年龄", "age");
    mappings.insert("性别", "gender");
    mappings.insert("身份证", "id_card");
    mappings.insert("身份证号", "id_card");
    mappings.insert("年龄", "age");
    mappings.insert("省份", "province");
    mappings.insert("城市", "city");
    mappings.insert("区县", "district");
    mappings.insert("邮编", "zipcode");
    mappings.insert("邮编", "postal_code");
    mappings.insert("网址", "website");
    mappings.insert("网站", "website");
    mappings.insert("链接", "url");
    mappings.insert("职位", "position");
    mappings.insert("职务", "job_title");
    mappings.insert("部门", "department");
    mappings.insert("备注", "note");
    mappings.insert("备注信息", "notes");
    mappings.insert("创建时间", "created_at");
    mappings.insert("更新时间", "updated_at");
    mappings.insert("订单号", "order_id");
    mappings.insert("订单编号", "order_number");
    mappings.insert("产品", "product");
    mappings.insert("商品", "product");
    mappings.insert("品牌", "brand");
    mappings.insert("规格", "specification");
    mappings.insert("型号", "model");
    mappings.insert("单位", "unit");
    mappings.insert("备注", "memo");
    mappings.insert("总额", "total");
    mappings.insert("合计", "total");
    mappings.insert("税额", "tax");
    mappings.insert("税率", "tax_rate");

    // 尝试完全匹配
    if let Some(&english) = mappings.get(label.trim()) {
        return english.to_string();
    }

    // 尝试包含匹配
    let label_trimmed = label.trim();
    for (cn, en) in &mappings {
        if label_trimmed.contains(cn) {
            return en.to_string();
        }
    }

    // 如果没有匹配，使用拼音首字母或生成随机字段名
    let name = label
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
        .replace(' ', "_");

    if name.is_empty() {
        format!("field_{}", chrono::Utc::now().timestamp() % 10000)
    } else if name.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
        format!("field_{}", name)
    } else {
        name
    }
}

/// 生成提取提示
fn generate_extraction_hint(field_label: &str, field_type: &str, additional: Option<&str>) -> String {
    let type_hint = match field_type {
        "phone" => "手机号码",
        "email" => "电子邮箱",
        "url" => "网址链接",
        "date" => "日期",
        "number" => "数字",
        _ => "文本",
    };

    let mut hint = format!("提取「{}」字段，类型为{}", field_label, type_hint);

    if let Some(add) = additional {
        if !add.is_empty() {
            hint.push_str(&format!("。附加要求：{}", add));
        }
    }

    hint
}

// ============ Tauri Commands ============

/// 获取项目的字段列表（不包括已删除的字段）
#[tauri::command]
pub async fn get_fields(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
) -> Result<Vec<FieldResponse>, String> {
    let fields = ProjectField::find()
        .filter(
            Condition::all()
                .add(field::Column::ProjectId.eq(project_id))
                .add(field::Column::IsDeleted.eq(false))
        )
        .order_by_asc(field::Column::DisplayOrder)
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(fields.into_iter().map(Into::into).collect())
}

/// 获取项目的所有字段（包括已删除的字段）
#[tauri::command]
pub async fn get_all_fields(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
) -> Result<Vec<FieldResponse>, String> {
    let fields = ProjectField::find()
        .filter(field::Column::ProjectId.eq(project_id))
        .order_by_asc(field::Column::DisplayOrder)
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(fields.into_iter().map(Into::into).collect())
}

/// 创建字段（支持恢复已删除的同名字段）
#[tauri::command]
pub async fn create_field(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    field_name: String,
    field_label: String,
    field_type: String,
    is_required: bool,  // 改为 bool 而不是 Option<bool>
    is_dedup_key: bool,  // 改为 bool 而不是 Option<bool>
    additional_requirement: Option<String>,
    validation_rule: Option<String>,
    extraction_hint: Option<String>,
) -> Result<FieldResponse, String> {
    // 第一时间打印日志，确认函数被调用
    eprintln!("===== create_field START =====");
    eprintln!("project_id: {}", project_id);
    eprintln!("field_name: {}", field_name);
    eprintln!("field_label: {}", field_label);
    eprintln!("field_type: {}", field_type);
    eprintln!("is_required: {:?}", is_required);
    eprintln!("is_dedup_key: {:?}", is_dedup_key);
    eprintln!("additional_requirement: {:?}", additional_requirement);
    eprintln!("validation_rule: {:?}", validation_rule);
    eprintln!("extraction_hint: {:?}", extraction_hint);
    eprintln!("===== create_field PARAMS END =====");

    tracing::info!("create_field called: project_id={}, field_name={}, field_label={}, field_type={}",
        project_id, field_name, field_label, field_type);

    // 验证必需参数
    let field_name = field_name.trim().to_string();
    let field_label = field_label.trim().to_string();

    if field_name.is_empty() {
        tracing::error!("create_field: field_name is empty");
        return Err("字段名不能为空".to_string());
    }
    if field_label.is_empty() {
        tracing::error!("create_field: field_label is empty");
        return Err("字段标签不能为空".to_string());
    }
    if field_type.trim().is_empty() {
        tracing::error!("create_field: field_type is empty");
        return Err("字段类型不能为空".to_string());
    }

    // 处理可选字段：空字符串转为 None
    let additional_requirement = additional_requirement.and_then(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) });
    let validation_rule = validation_rule.and_then(|s| if s.trim().is_empty() { None } else { Some(s) });
    let extraction_hint = extraction_hint.and_then(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) });

    tracing::info!("create_field: processed values - additional_requirement={:?}, validation_rule={:?}, extraction_hint={:?}",
        additional_requirement, validation_rule, extraction_hint);

    // 检查是否存在同名的已删除字段
    let deleted_field = ProjectField::find()
        .filter(
            Condition::all()
                .add(field::Column::ProjectId.eq(project_id))
                .add(field::Column::FieldName.eq(&field_name))
                .add(field::Column::IsDeleted.eq(true))
        )
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    if let Some(model) = deleted_field {
        tracing::info!("create_field: restoring deleted field id={}", model.id);
        // 恢复已删除的字段
        let mut active: field::ActiveModel = model.into();
        active.is_deleted = Set(false);
        active.deleted_at = Set(None);
        active.field_label = Set(field_label);
        active.field_type = Set(field_type);
        active.is_required = Set(is_required);
        active.is_dedup_key = Set(is_dedup_key);
        active.additional_requirement = Set(additional_requirement);
        active.validation_rule = Set(validation_rule);
        active.extraction_hint = Set(extraction_hint);

        tracing::info!("create_field: updating restored field...");
        let result = active
            .update(db.inner().as_ref())
            .await
            .map_err(|e| {
                tracing::error!("create_field: update failed - {}", e);
                format!("数据库错误: {}", e)
            })?;

        tracing::info!("create_field: field restored successfully");
        return Ok(result.into());
    }

    // 检查是否存在同名的未删除字段
    tracing::info!("create_field: checking for existing field...");
    let existing = ProjectField::find()
        .filter(
            Condition::all()
                .add(field::Column::ProjectId.eq(project_id))
                .add(field::Column::FieldName.eq(&field_name))
                .add(field::Column::IsDeleted.eq(false))
        )
        .one(db.inner().as_ref())
        .await
        .map_err(|e| {
            tracing::error!("create_field: check existing failed - {}", e);
            format!("数据库错误: {}", e)
        })?;

    if existing.is_some() {
        tracing::error!("create_field: field '{}' already exists", field_name);
        return Err(format!("字段 '{}' 已存在", field_name));
    }

    // 获取最大 display_order
    tracing::info!("create_field: getting max display_order...");
    let max_order = ProjectField::find()
        .filter(field::Column::ProjectId.eq(project_id))
        .order_by_desc(field::Column::DisplayOrder)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| {
            tracing::error!("create_field: get max_order failed - {}", e);
            format!("数据库错误: {}", e)
        })?
        .map(|f| f.display_order)
        .unwrap_or(-1);

    tracing::info!("create_field: creating new field with display_order={}", max_order + 1);

    let now = chrono::Utc::now();
    let new_field = field::ActiveModel {
        project_id: Set(project_id),
        field_name: Set(field_name.clone()),
        field_label: Set(field_label.clone()),
        field_type: Set(field_type.clone()),
        is_required: Set(is_required),
        is_dedup_key: Set(is_dedup_key),
        is_deleted: Set(false),
        additional_requirement: Set(additional_requirement.clone()),
        validation_rule: Set(validation_rule.clone()),
        extraction_hint: Set(extraction_hint.clone()),
        display_order: Set(max_order + 1),
        created_at: Set(now),
        deleted_at: Set(None),
        ..Default::default()
    };

    tracing::info!("create_field: inserting into database...");
    let result = new_field
        .insert(db.inner().as_ref())
        .await
        .map_err(|e| {
            tracing::error!("create_field: insert failed - {}", e);
            format!("数据库错误: {}", e)
        })?;

    tracing::info!("create_field: field created successfully with id={}", result.id);
    Ok(result.into())
}

/// 更新字段
#[tauri::command]
pub async fn update_field(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
    field_name: Option<String>,
    field_label: Option<String>,
    field_type: Option<String>,
    is_required: Option<bool>,
    is_dedup_key: Option<bool>,
    additional_requirement: Option<String>,
    validation_rule: Option<String>,
    extraction_hint: Option<String>,
    display_order: Option<i32>,
) -> Result<FieldResponse, String> {
    tracing::info!("update_field called: id={}, field_name={:?}, field_label={:?}, field_type={:?}",
        id, field_name, field_label, field_type);

    // 验证 ID 有效性
    if id <= 0 {
        tracing::error!("update_field: invalid id {}", id);
        return Err(format!("无效的字段 ID: {}（ID 必须为正整数）", id));
    }

    tracing::info!("update_field: finding field by id...");
    let field = ProjectField::find_by_id(id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| {
            tracing::error!("update_field: find failed - {}", e);
            format!("数据库错误: {}", e)
        })?
        .ok_or_else(|| {
            tracing::error!("update_field: field {} not found", id);
            format!("字段 {} 不存在", id)
        })?;

    tracing::info!("update_field: found field, converting to active model...");
    let mut active: field::ActiveModel = field.into();

    if let Some(name) = field_name {
        let name = name.trim().to_string();
        if !name.is_empty() {
            tracing::info!("update_field: setting field_name={}", name);
            active.field_name = Set(name);
        }
    }
    if let Some(label) = field_label {
        let label = label.trim().to_string();
        if !label.is_empty() {
            tracing::info!("update_field: setting field_label={}", label);
            active.field_label = Set(label);
        }
    }
    if let Some(ft) = field_type {
        if !ft.trim().is_empty() {
            tracing::info!("update_field: setting field_type={}", ft);
            active.field_type = Set(ft);
        }
    }
    if let Some(req) = is_required {
        tracing::info!("update_field: setting is_required={}", req);
        active.is_required = Set(req);
    }
    if let Some(dedup) = is_dedup_key {
        tracing::info!("update_field: setting is_dedup_key={}", dedup);
        active.is_dedup_key = Set(dedup);
    }

    // 处理可选字段：空字符串转为 None
    let processed_additional = additional_requirement.and_then(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) });
    let processed_validation = validation_rule.and_then(|s| if s.trim().is_empty() { None } else { Some(s) });
    let processed_extraction = extraction_hint.and_then(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) });

    tracing::info!("update_field: processed optional fields - additional={:?}, validation={:?}, extraction={:?}",
        processed_additional, processed_validation, processed_extraction);

    active.additional_requirement = Set(processed_additional);
    active.validation_rule = Set(processed_validation);
    active.extraction_hint = Set(processed_extraction);

    if let Some(order) = display_order {
        active.display_order = Set(order);
    }

    tracing::info!("update_field: updating database...");
    let result = active
        .update(db.inner().as_ref())
        .await
        .map_err(|e| {
            tracing::error!("update_field: update failed - {}", e);
            format!("数据库错误: {}", e)
        })?;

    tracing::info!("update_field: field updated successfully");
    Ok(result.into())
}

/// 软删除字段
#[tauri::command]
pub async fn delete_field(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
) -> Result<(), String> {
    eprintln!("===== delete_field START: id={} =====", id);

    let field = ProjectField::find_by_id(id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| {
            eprintln!("delete_field: find error - {}", e);
            format!("数据库错误: {}", e)
        })?
        .ok_or_else(|| {
            eprintln!("delete_field: field {} not found", id);
            format!("字段 {} 不存在", id)
        })?;

    eprintln!("delete_field: found field '{}', setting is_deleted=true", field.field_name);

    let mut active: field::ActiveModel = field.into();
    active.is_deleted = Set(true);
    active.deleted_at = Set(Some(chrono::Utc::now()));

    active
        .update(db.inner().as_ref())
        .await
        .map_err(|e| {
            eprintln!("delete_field: update error - {}", e);
            format!("数据库错误: {}", e)
        })?;

    eprintln!("===== delete_field SUCCESS =====");
    Ok(())
}

/// 恢复已删除的字段
#[tauri::command]
pub async fn restore_field(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
) -> Result<FieldResponse, String> {
    let field = ProjectField::find_by_id(id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("字段 {} 不存在", id))?;

    if !field.is_deleted {
        return Err("该字段未被删除".to_string());
    }

    let mut active: field::ActiveModel = field.into();
    active.is_deleted = Set(false);
    active.deleted_at = Set(None);

    let result = active
        .update(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.into())
}

/// 生成本地验证规则和提取提示（不调用 AI）
#[tauri::command]
pub async fn generate_field_metadata(
    field_label: String,
    field_type: String,
    additional_requirement: Option<String>,
) -> Result<MetadataResponse, String> {
    // 生成字段名
    let field_name = generate_field_name(&field_label);

    // 获取验证规则
    let validation_rule = get_validation_rule(&field_type);

    // 生成提取提示
    let extraction_hint = generate_extraction_hint(
        &field_label,
        &field_type,
        additional_requirement.as_deref(),
    );

    Ok(MetadataResponse {
        field_name,
        validation_rule,
        extraction_hint,
    })
}
