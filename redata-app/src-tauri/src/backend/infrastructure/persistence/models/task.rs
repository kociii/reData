// ProcessingTask 模型

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "processing_tasks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String, // UUID

    #[sea_orm(indexed)]
    pub project_id: i32,

    /// pending, processing, paused, completed, cancelled
    #[sea_orm(indexed)]
    pub status: String,

    #[sea_orm(default_value = "0")]
    pub total_files: i32,

    #[sea_orm(default_value = "0")]
    pub processed_files: i32,

    #[sea_orm(default_value = "0")]
    pub total_rows: i32,

    #[sea_orm(default_value = "0")]
    pub processed_rows: i32,

    #[sea_orm(default_value = "0")]
    pub success_count: i32,

    #[sea_orm(default_value = "0")]
    pub error_count: i32,

    pub batch_number: Option<String>,

    pub created_at: DateTimeUtc,

    pub updated_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
