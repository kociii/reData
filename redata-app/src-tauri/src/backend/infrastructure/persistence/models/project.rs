// Project 模型

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique, indexed)]
    pub name: String,

    pub description: Option<String>,

    #[sea_orm(default_value = "true")]
    pub dedup_enabled: bool,

    /// JSON 字符串，存储去重字段列表
    pub dedup_fields: Option<String>,

    #[sea_orm(default_value = "skip")]
    pub dedup_strategy: String, // skip, update, merge

    /// 所属分组 ID
    pub group_id: Option<i32>,

    pub created_at: DateTimeUtc,

    pub updated_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project_group::Entity",
        from = "Column::GroupId",
        to = "super::project_group::Column::Id"
    )]
    Group,
}

impl Related<super::project_group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Group.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// 获取去重字段列表
    pub fn get_dedup_fields(&self) -> Vec<String> {
        self.dedup_fields
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    /// 设置去重字段列表
    pub fn set_dedup_fields(&mut self, fields: Vec<String>) {
        self.dedup_fields = if fields.is_empty() {
            None
        } else {
            serde_json::to_string(&fields).ok()
        };
    }
}
