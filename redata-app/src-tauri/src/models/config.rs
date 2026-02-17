use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub id: Option<i64>,
    pub name: String,
    pub api_url: String,
    pub model_name: String,
    pub api_key: String,
    pub temperature: f32,
    pub max_tokens: i32,
    pub is_default: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl AiConfig {
    pub fn new(name: String, api_url: String, model_name: String, api_key: String) -> Self {
        Self {
            id: None,
            name,
            api_url,
            model_name,
            api_key,
            temperature: 0.7,
            max_tokens: 1000,
            is_default: false,
            created_at: None,
            updated_at: None,
        }
    }
}
