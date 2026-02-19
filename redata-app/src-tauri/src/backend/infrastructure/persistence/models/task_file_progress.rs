// TaskFileProgress 模型 - 任务文件进度持久化

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "task_file_progress")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(indexed)]
    pub task_id: String,

    pub file_name: String,

    /// waiting, processing, done, error
    pub file_phase: String,

    /// Sheet 名称（如果是 Sheet 级别记录）
    pub sheet_name: Option<String>,

    /// waiting, ai_analyzing, importing, done, error
    pub sheet_phase: Option<String>,

    /// AI 置信度 (0-1)
    pub ai_confidence: Option<f32>,

    /// 映射成功的字段数
    pub mapping_count: Option<i32>,

    #[sea_orm(default_value = "0")]
    pub success_count: i32,

    #[sea_orm(default_value = "0")]
    pub error_count: i32,

    #[sea_orm(default_value = "0")]
    pub total_rows: i32,

    pub error_message: Option<String>,

    pub created_at: DateTimeUtc,

    pub updated_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::task::Entity",
        from = "Column::TaskId",
        to = "super::task::Column::Id"
    )]
    Task,
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
