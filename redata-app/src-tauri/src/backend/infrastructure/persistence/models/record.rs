// ProjectRecord 模型 - JSON 统一存储方案
// data 字段以 field_id 为 key 存储动态字段值，如 {"3": "张三", "5": "13800138000"}

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "project_records")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(indexed)]
    pub project_id: i32,

    /// JSON 格式的动态字段数据，key 为 field_id
    #[sea_orm(column_type = "Text")]
    pub data: String,

    pub source_file: Option<String>,
    pub source_sheet: Option<String>,
    pub row_number: Option<i32>,
    pub batch_number: Option<String>,

    #[sea_orm(default_value = "success")]
    pub status: String,

    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
