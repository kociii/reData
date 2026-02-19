// ProjectGroup 模型 - 项目分组

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "project_groups")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub name: String,

    /// 父分组 ID（支持嵌套）
    pub parent_id: Option<i32>,

    /// 分组颜色
    pub color: Option<String>,

    /// 分组图标
    pub icon: Option<String>,

    /// 排序顺序
    #[sea_orm(default_value = "0")]
    pub sort_order: i32,

    pub created_at: DateTimeUtc,

    pub updated_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::Id"
    )]
    Parent,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        super::project::Relation::Group.def().rev()
    }
}

impl ActiveModelBehavior for ActiveModel {}
