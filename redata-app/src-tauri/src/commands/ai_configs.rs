// AI 配置管理 Tauri Commands
//
// 实现 AI 配置的 CRUD 操作、加密存储和连接测试

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::backend::infrastructure::{
    config::{encrypt, decrypt},
    persistence::models::{ai_config, AiConfig},
};

// ============ 请求/响应结构 ============

/// AI 配置创建请求
#[derive(Debug, Deserialize)]
pub struct CreateAiConfigRequest {
    pub name: String,
    pub api_url: String,
    pub model_name: String,
    pub api_key: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: i32,
    #[serde(default)]
    pub is_default: bool,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> i32 {
    1000
}

/// AI 配置更新请求
#[derive(Debug, Deserialize)]
pub struct UpdateAiConfigRequest {
    pub name: Option<String>,
    pub api_url: Option<String>,
    pub model_name: Option<String>,
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub is_default: Option<bool>,
}

/// AI 配置响应（API Key 会被掩码处理）
#[derive(Debug, Serialize, Clone)]
pub struct AiConfigResponse {
    pub id: i32,
    pub name: String,
    pub api_url: String,
    pub model_name: String,
    pub api_key: String,  // 掩码后的 API Key
    pub temperature: f32,
    pub max_tokens: i32,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// 连接测试响应
#[derive(Debug, Serialize)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub message: String,
    pub response: Option<String>,
}

// ============ 辅助函数 ============

/// 掩码 API Key（只显示前 4 位和后 4 位）
fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 8 {
        return "*".repeat(api_key.len());
    }
    let start = &api_key[..4];
    let end = &api_key[api_key.len() - 4..];
    format!("{}****{}", start, end)
}

impl From<ai_config::Model> for AiConfigResponse {
    fn from(model: ai_config::Model) -> Self {
        // 解密并掩码 API Key
        let masked_key = decrypt(&model.api_key)
            .map(|k| mask_api_key(&k))
            .unwrap_or_else(|_| "****".to_string());

        Self {
            id: model.id,
            name: model.name,
            api_url: model.api_url,
            model_name: model.model_name,
            api_key: masked_key,
            temperature: model.temperature,
            max_tokens: model.max_tokens,
            is_default: model.is_default,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

// ============ Tauri Commands ============

/// 获取所有 AI 配置
#[tauri::command]
pub async fn get_ai_configs(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
) -> Result<Vec<AiConfigResponse>, String> {
    let configs = AiConfig::find()
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(configs.into_iter().map(Into::into).collect())
}

/// 获取单个 AI 配置
#[tauri::command]
pub async fn get_ai_config(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
) -> Result<AiConfigResponse, String> {
    let config = AiConfig::find_by_id(id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("AI 配置 {} 不存在", id))?;

    Ok(config.into())
}

/// 获取默认 AI 配置
#[tauri::command]
pub async fn get_default_ai_config(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
) -> Result<AiConfigResponse, String> {
    let config = AiConfig::find()
        .filter(ai_config::Column::IsDefault.eq(true))
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| "未找到默认 AI 配置".to_string())?;

    Ok(config.into())
}

/// 创建 AI 配置
#[tauri::command]
pub async fn create_ai_config(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    name: String,
    api_url: String,
    model_name: String,
    api_key: String,
    temperature: Option<f32>,
    max_tokens: Option<i32>,
    is_default: Option<bool>,
) -> Result<AiConfigResponse, String> {
    // 检查名称是否已存在
    let existing = AiConfig::find()
        .filter(ai_config::Column::Name.eq(&name))
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    if existing.is_some() {
        return Err(format!("AI 配置 '{}' 已存在", name));
    }

    // 加密 API Key
    let encrypted_key = encrypt(&api_key)
        .map_err(|e| format!("加密失败: {}", e))?;

    let is_default_val = is_default.unwrap_or(false);

    // 如果设置为默认，需要取消其他默认配置
    if is_default_val {
        let txn = db.inner().as_ref().begin().await
            .map_err(|e| format!("数据库错误: {}", e))?;

        // 取消其他默认配置
        let others = AiConfig::find()
            .filter(ai_config::Column::IsDefault.eq(true))
            .all(&txn)
            .await
            .map_err(|e| format!("数据库错误: {}", e))?;

        for other in others {
            let mut active: ai_config::ActiveModel = other.into();
            active.is_default = Set(false);
            active.updated_at = Set(Some(chrono::Utc::now()));
            active.update(&txn).await
                .map_err(|e| format!("数据库错误: {}", e))?;
        }

        let now = chrono::Utc::now();
        let new_config = ai_config::ActiveModel {
            name: Set(name),
            api_url: Set(api_url),
            model_name: Set(model_name),
            api_key: Set(encrypted_key),
            temperature: Set(temperature.unwrap_or(0.7)),
            max_tokens: Set(max_tokens.unwrap_or(1000)),
            is_default: Set(true),
            created_at: Set(now),
            updated_at: Set(None),
            ..Default::default()
        };

        let result = new_config.insert(&txn).await
            .map_err(|e| format!("数据库错误: {}", e))?;

        txn.commit().await
            .map_err(|e| format!("数据库错误: {}", e))?;

        return Ok(result.into());
    }

    let now = chrono::Utc::now();
    let new_config = ai_config::ActiveModel {
        name: Set(name),
        api_url: Set(api_url),
        model_name: Set(model_name),
        api_key: Set(encrypted_key),
        temperature: Set(temperature.unwrap_or(0.7)),
        max_tokens: Set(max_tokens.unwrap_or(1000)),
        is_default: Set(false),
        created_at: Set(now),
        updated_at: Set(None),
        ..Default::default()
    };

    let result = new_config
        .insert(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.into())
}

/// 更新 AI 配置
#[tauri::command]
pub async fn update_ai_config(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
    name: Option<String>,
    api_url: Option<String>,
    model_name: Option<String>,
    api_key: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<i32>,
    is_default: Option<bool>,
) -> Result<AiConfigResponse, String> {
    let config = AiConfig::find_by_id(id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("AI 配置 {} 不存在", id))?;

    let mut active: ai_config::ActiveModel = config.into();

    if let Some(n) = name {
        // 检查新名称是否已被其他配置使用
        let existing = AiConfig::find()
            .filter(ai_config::Column::Name.eq(&n))
            .filter(ai_config::Column::Id.ne(id))
            .one(db.inner().as_ref())
            .await
            .map_err(|e| format!("数据库错误: {}", e))?;

        if existing.is_some() {
            return Err(format!("AI 配置名称 '{}' 已被使用", n));
        }
        active.name = Set(n);
    }
    if let Some(url) = api_url {
        active.api_url = Set(url);
    }
    if let Some(model) = model_name {
        active.model_name = Set(model);
    }
    if let Some(key) = api_key {
        let encrypted = encrypt(&key)
            .map_err(|e| format!("加密失败: {}", e))?;
        active.api_key = Set(encrypted);
    }
    if let Some(temp) = temperature {
        active.temperature = Set(temp);
    }
    if let Some(tokens) = max_tokens {
        active.max_tokens = Set(tokens);
    }
    if let Some(default) = is_default {
        active.is_default = Set(default);
    }

    active.updated_at = Set(Some(chrono::Utc::now()));

    let result = active
        .update(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.into())
}

/// 删除 AI 配置
#[tauri::command]
pub async fn delete_ai_config(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
) -> Result<(), String> {
    let config = AiConfig::find_by_id(id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("AI 配置 {} 不存在", id))?;

    let active: ai_config::ActiveModel = config.into();
    active
        .delete(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(())
}

/// 设置默认 AI 配置
#[tauri::command]
pub async fn set_default_ai_config(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
) -> Result<AiConfigResponse, String> {
    // 使用事务确保原子性
    let txn = db.inner().as_ref().begin().await
        .map_err(|e| format!("数据库错误: {}", e))?;

    // 检查目标配置是否存在
    let config = AiConfig::find_by_id(id)
        .one(&txn)
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("AI 配置 {} 不存在", id))?;

    // 取消所有默认配置
    let all_defaults = AiConfig::find()
        .filter(ai_config::Column::IsDefault.eq(true))
        .all(&txn)
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    for default_config in all_defaults {
        let mut active: ai_config::ActiveModel = default_config.into();
        active.is_default = Set(false);
        active.updated_at = Set(Some(chrono::Utc::now()));
        active.update(&txn).await
            .map_err(|e| format!("数据库错误: {}", e))?;
    }

    // 设置新的默认配置
    let mut active: ai_config::ActiveModel = config.into();
    active.is_default = Set(true);
    active.updated_at = Set(Some(chrono::Utc::now()));
    let result = active.update(&txn).await
        .map_err(|e| format!("数据库错误: {}", e))?;

    txn.commit().await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.into())
}

/// 测试 AI 连接
#[tauri::command]
pub async fn test_ai_connection(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    id: i32,
) -> Result<TestConnectionResponse, String> {
    let config = AiConfig::find_by_id(id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("AI 配置 {} 不存在", id))?;

    // 解密 API Key
    let api_key = decrypt(&config.api_key)
        .map_err(|e| format!("解密失败: {}", e))?;

    // 调用 AI API 进行测试
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/chat/completions", config.api_url.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": config.model_name,
            "messages": [{"role": "user", "content": "Reply: OK"}],
            "max_tokens": 2,
        }))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|_| "无法读取响应".to_string());

            if status.is_success() {
                Ok(TestConnectionResponse {
                    success: true,
                    message: "连接成功".to_string(),
                    response: Some(body),
                })
            } else {
                Ok(TestConnectionResponse {
                    success: false,
                    message: format!("API 返回错误: {}", status),
                    response: Some(body),
                })
            }
        }
        Err(e) => {
            Ok(TestConnectionResponse {
                success: false,
                message: format!("连接失败: {}", e),
                response: None,
            })
        }
    }
}

/// 获取解密后的 API Key（内部使用，不暴露给前端）
pub fn get_decrypted_api_key(config: &ai_config::Model) -> Result<String, String> {
    decrypt(&config.api_key).map_err(|e| format!("解密失败: {}", e))
}
