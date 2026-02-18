// AiConfig 模型

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ai_configs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique)]
    pub name: String,

    pub api_url: String,

    pub model_name: String,

    /// 加密存储的 API 密钥
    pub api_key: String,

    #[sea_orm(default_value = "0.7")]
    pub temperature: f32,

    #[sea_orm(default_value = "1000")]
    pub max_tokens: i32,

    #[sea_orm(default_value = "false")]
    pub is_default: bool,

    pub created_at: DateTimeUtc,

    pub updated_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
