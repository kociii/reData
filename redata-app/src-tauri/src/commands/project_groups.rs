// 项目分组管理 Tauri Commands

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set, Statement,
};
use serde::Serialize;
use std::sync::Arc;

use crate::backend::infrastructure::persistence::models::{
    project, project_group, Project, ProjectGroup,
};

// ============ 响应结构 ============

#[derive(Debug, Serialize)]
pub struct ProjectGroupResponse {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub project_count: i32,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GroupWithChildren {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub project_count: i32,
    pub children: Vec<GroupWithChildren>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl From<project_group::Model> for ProjectGroupResponse {
    fn from(m: project_group::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            parent_id: m.parent_id,
            color: m.color,
            icon: m.icon,
            sort_order: m.sort_order,
            project_count: 0, // 需要单独计算
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.map(|t| t.to_rfc3339()),
        }
    }
}

// ============ Tauri Commands ============

/// 获取所有分组（带层级结构）
#[tauri::command]
pub async fn get_project_groups(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
) -> Result<Vec<GroupWithChildren>, String> {
    // 获取所有分组
    let groups = ProjectGroup::find()
        .order_by(project_group::Column::SortOrder, sea_orm::Order::Asc)
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    // 获取每个分组的项目数量
    let projects = Project::find()
        .filter(project::Column::GroupId.is_not_null())
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    let mut count_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
    for p in projects {
        if let Some(gid) = p.group_id {
            *count_map.entry(gid).or_insert(0) += 1;
        }
    }

    // 构建树形结构
    fn build_tree(
        groups: &[project_group::Model],
        count_map: &std::collections::HashMap<i32, i32>,
        parent_id: Option<i32>,
    ) -> Vec<GroupWithChildren> {
        groups
            .iter()
            .filter(|g| g.parent_id == parent_id)
            .map(|g| GroupWithChildren {
                id: g.id,
                name: g.name.clone(),
                parent_id: g.parent_id,
                color: g.color.clone(),
                icon: g.icon.clone(),
                sort_order: g.sort_order,
                project_count: *count_map.get(&g.id).unwrap_or(&0),
                children: build_tree(groups, count_map, Some(g.id)),
                created_at: g.created_at.to_rfc3339(),
                updated_at: g.updated_at.map(|t| t.to_rfc3339()),
            })
            .collect()
    }

    Ok(build_tree(&groups, &count_map, None))
}

/// 获取所有分组（扁平列表）
#[tauri::command]
pub async fn get_project_groups_flat(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
) -> Result<Vec<ProjectGroupResponse>, String> {
    let groups = ProjectGroup::find()
        .order_by(project_group::Column::SortOrder, sea_orm::Order::Asc)
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    // 获取每个分组的项目数量
    let projects = Project::find()
        .filter(project::Column::GroupId.is_not_null())
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    let mut count_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
    for p in projects {
        if let Some(gid) = p.group_id {
            *count_map.entry(gid).or_insert(0) += 1;
        }
    }

    Ok(groups
        .into_iter()
        .map(|g| ProjectGroupResponse {
            id: g.id,
            name: g.name,
            parent_id: g.parent_id,
            color: g.color,
            icon: g.icon,
            sort_order: g.sort_order,
            project_count: *count_map.get(&g.id).unwrap_or(&0),
            created_at: g.created_at.to_rfc3339(),
            updated_at: g.updated_at.map(|t| t.to_rfc3339()),
        })
        .collect())
}

/// 创建分组
#[tauri::command]
pub async fn create_project_group(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    name: String,
    parent_id: Option<i32>,
    color: Option<String>,
    icon: Option<String>,
) -> Result<ProjectGroupResponse, String> {
    // 获取当前最大排序值
    let max_order = ProjectGroup::find()
        .filter(project_group::Column::ParentId.eq(parent_id))
        .all(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .iter()
        .map(|g| g.sort_order)
        .max()
        .unwrap_or(-1);

    let new_group = project_group::ActiveModel {
        name: Set(name),
        parent_id: Set(parent_id),
        color: Set(color),
        icon: Set(icon),
        sort_order: Set(max_order + 1),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(None),
        ..Default::default()
    };

    let result = new_group
        .insert(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.into())
}

/// 更新分组
#[tauri::command]
pub async fn update_project_group(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    group_id: i32,
    name: Option<String>,
    color: Option<String>,
    icon: Option<String>,
    sort_order: Option<i32>,
) -> Result<ProjectGroupResponse, String> {
    let group = ProjectGroup::find_by_id(group_id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("分组 {} 不存在", group_id))?;

    let mut active: project_group::ActiveModel = group.into();
    if let Some(n) = name {
        active.name = Set(n);
    }
    if color.is_some() {
        active.color = Set(color);
    }
    if icon.is_some() {
        active.icon = Set(icon);
    }
    if let Some(order) = sort_order {
        active.sort_order = Set(order);
    }
    active.updated_at = Set(Some(chrono::Utc::now()));

    let result = active
        .update(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(result.into())
}

/// 删除分组
#[tauri::command]
pub async fn delete_project_group(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    group_id: i32,
) -> Result<(), String> {
    // 批量更新：将该分组下的项目的 group_id 置空
    db.inner()
        .as_ref()
        .execute(Statement::from_sql_and_values(
            db.inner().as_ref().get_database_backend(),
            "UPDATE projects SET group_id = NULL WHERE group_id = ?",
            vec![group_id.into()],
        ))
        .await
        .map_err(|e| format!("更新项目失败: {}", e))?;

    // 批量更新：将子分组的 parent_id 置空
    db.inner()
        .as_ref()
        .execute(Statement::from_sql_and_values(
            db.inner().as_ref().get_database_backend(),
            "UPDATE project_groups SET parent_id = NULL WHERE parent_id = ?",
            vec![group_id.into()],
        ))
        .await
        .map_err(|e| format!("更新子分组失败: {}", e))?;

    // 删除分组
    ProjectGroup::delete_by_id(group_id)
        .exec(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(())
}

/// 移动项目到分组
#[tauri::command]
pub async fn move_project_to_group(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_id: i32,
    group_id: Option<i32>,
) -> Result<(), String> {
    let project = Project::find_by_id(project_id)
        .one(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?
        .ok_or_else(|| format!("项目 {} 不存在", project_id))?;

    let mut active: project::ActiveModel = project.into();
    active.group_id = Set(group_id);
    active.updated_at = Set(Some(chrono::Utc::now()));

    active
        .update(db.inner().as_ref())
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;

    Ok(())
}

/// 批量移动项目
#[tauri::command]
pub async fn batch_move_projects(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    project_ids: Vec<i32>,
    group_id: Option<i32>,
) -> Result<u64, String> {
    let mut count = 0u64;
    for pid in project_ids {
        if let Some(project) = Project::find_by_id(pid)
            .one(db.inner().as_ref())
            .await
            .map_err(|e| format!("数据库错误: {}", e))?
        {
            let mut active: project::ActiveModel = project.into();
            active.group_id = Set(group_id);
            active.updated_at = Set(Some(chrono::Utc::now()));
            active
                .update(db.inner().as_ref())
                .await
                .map_err(|e| format!("数据库错误: {}", e))?;
            count += 1;
        }
    }
    Ok(count)
}

/// 更新分组排序
#[tauri::command]
pub async fn reorder_project_groups(
    db: tauri::State<'_, Arc<DatabaseConnection>>,
    group_orders: Vec<(i32, i32)>, // (group_id, sort_order)
) -> Result<(), String> {
    for (group_id, sort_order) in group_orders {
        if let Some(group) = ProjectGroup::find_by_id(group_id)
            .one(db.inner().as_ref())
            .await
            .map_err(|e| format!("数据库错误: {}", e))?
        {
            let mut active: project_group::ActiveModel = group.into();
            active.sort_order = Set(sort_order);
            active.updated_at = Set(Some(chrono::Utc::now()));
            active
                .update(db.inner().as_ref())
                .await
                .map_err(|e| format!("数据库错误: {}", e))?;
        }
    }
    Ok(())
}
