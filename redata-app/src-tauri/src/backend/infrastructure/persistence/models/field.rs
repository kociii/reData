// ProjectField 模型

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "project_fields")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(indexed)]
    pub project_id: i32,

    pub field_name: String,

    pub field_label: String,

    /// text, number, email, phone, date, url
    pub field_type: String,

    #[sea_orm(default_value = "false")]
    pub is_required: bool,

    /// 是否参与去重
    #[sea_orm(default_value = "false")]
    pub is_dedup_key: bool,

    /// 软删除标记
    #[sea_orm(default_value = "false")]
    pub is_deleted: bool,

    pub additional_requirement: Option<String>,

    pub validation_rule: Option<String>,

    pub extraction_hint: Option<String>,

    #[sea_orm(default_value = "0")]
    pub display_order: i32,

    pub created_at: DateTimeUtc,

    /// 删除时间
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
