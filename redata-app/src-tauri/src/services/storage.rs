use crate::db;
use crate::models::{Project, ProjectField, AiConfig, ProcessingTask};
use rusqlite::{params, Result};

pub struct StorageService;

impl StorageService {
    // ========== 项目管理 ==========
    
    pub fn create_project(project: &Project) -> Result<i64> {
        let conn = db::get_connection()?;
        
        let dedup_fields_json = project.dedup_fields.as_ref()
            .map(|fields| serde_json::to_string(fields).unwrap_or_default());
        
        conn.execute(
            "INSERT INTO projects (name, description, dedup_enabled, dedup_fields, dedup_strategy)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &project.name,
                &project.description,
                project.dedup_enabled as i32,
                dedup_fields_json,
                &project.dedup_strategy,
            ],
        )?;
        
        Ok(conn.last_insert_rowid())
    }
    
    pub fn get_project(id: i64) -> Result<Option<Project>> {
        let conn = db::get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, name, description, dedup_enabled, dedup_fields, dedup_strategy, 
                    created_at, updated_at
             FROM projects WHERE id = ?1"
        )?;
        
        let project = stmt.query_row(params![id], |row| {
            let dedup_fields_str: Option<String> = row.get(4)?;
            let dedup_fields = dedup_fields_str
                .and_then(|s| serde_json::from_str(&s).ok());
            
            Ok(Project {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                dedup_enabled: row.get::<_, i32>(3)? != 0,
                dedup_fields,
                dedup_strategy: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        }).optional()?;
        
        Ok(project)
    }
    
    pub fn list_projects() -> Result<Vec<Project>> {
        let conn = db::get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, name, description, dedup_enabled, dedup_fields, dedup_strategy,
                    created_at, updated_at
             FROM projects ORDER BY created_at DESC"
        )?;
        
        let projects = stmt.query_map([], |row| {
            let dedup_fields_str: Option<String> = row.get(4)?;
            let dedup_fields = dedup_fields_str
                .and_then(|s| serde_json::from_str(&s).ok());
            
            Ok(Project {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                dedup_enabled: row.get::<_, i32>(3)? != 0,
                dedup_fields,
                dedup_strategy: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        
        Ok(projects)
    }
    
    pub fn update_project(project: &Project) -> Result<()> {
        let conn = db::get_connection()?;
        
        let dedup_fields_json = project.dedup_fields.as_ref()
            .map(|fields| serde_json::to_string(fields).unwrap_or_default());
        
        conn.execute(
            "UPDATE projects 
             SET name = ?1, description = ?2, dedup_enabled = ?3, 
                 dedup_fields = ?4, dedup_strategy = ?5, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?6",
            params![
                &project.name,
                &project.description,
                project.dedup_enabled as i32,
                dedup_fields_json,
                &project.dedup_strategy,
                project.id,
            ],
        )?;
        
        Ok(())
    }
    
    pub fn delete_project(id: i64) -> Result<()> {
        let conn = db::get_connection()?;
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }
    
    // ========== 字段管理 ==========
    
    pub fn create_field(field: &ProjectField) -> Result<i64> {
        let conn = db::get_connection()?;
        
        conn.execute(
            "INSERT INTO project_fields 
             (project_id, field_name, field_label, field_type, is_required, 
              validation_rule, extraction_hint, display_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                field.project_id,
                &field.field_name,
                &field.field_label,
                &field.field_type,
                field.is_required as i32,
                &field.validation_rule,
                &field.extraction_hint,
                field.display_order,
            ],
        )?;
        
        Ok(conn.last_insert_rowid())
    }
    
    pub fn list_fields(project_id: i64) -> Result<Vec<ProjectField>> {
        let conn = db::get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, project_id, field_name, field_label, field_type, is_required,
                    validation_rule, extraction_hint, display_order, created_at
             FROM project_fields WHERE project_id = ?1 ORDER BY display_order"
        )?;
        
        let fields = stmt.query_map(params![project_id], |row| {
            Ok(ProjectField {
                id: Some(row.get(0)?),
                project_id: row.get(1)?,
                field_name: row.get(2)?,
                field_label: row.get(3)?,
                field_type: row.get(4)?,
                is_required: row.get::<_, i32>(5)? != 0,
                validation_rule: row.get(6)?,
                extraction_hint: row.get(7)?,
                display_order: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        
        Ok(fields)
    }
    
    pub fn update_field(field: &ProjectField) -> Result<()> {
        let conn = db::get_connection()?;
        
        conn.execute(
            "UPDATE project_fields 
             SET field_name = ?1, field_label = ?2, field_type = ?3, is_required = ?4,
                 validation_rule = ?5, extraction_hint = ?6, display_order = ?7
             WHERE id = ?8",
            params![
                &field.field_name,
                &field.field_label,
                &field.field_type,
                field.is_required as i32,
                &field.validation_rule,
                &field.extraction_hint,
                field.display_order,
                field.id,
            ],
        )?;
        
        Ok(())
    }
    
    pub fn delete_field(id: i64) -> Result<()> {
        let conn = db::get_connection()?;
        conn.execute("DELETE FROM project_fields WHERE id = ?1", params![id])?;
        Ok(())
    }
    
    // ========== AI 配置管理 ==========
    
    pub fn create_ai_config(config: &AiConfig) -> Result<i64> {
        let conn = db::get_connection()?;
        
        conn.execute(
            "INSERT INTO ai_configs 
             (name, api_url, model_name, api_key, temperature, max_tokens, is_default)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &config.name,
                &config.api_url,
                &config.model_name,
                &config.api_key,
                config.temperature,
                config.max_tokens,
                config.is_default as i32,
            ],
        )?;
        
        Ok(conn.last_insert_rowid())
    }
    
    pub fn list_ai_configs() -> Result<Vec<AiConfig>> {
        let conn = db::get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, name, api_url, model_name, api_key, temperature, max_tokens,
                    is_default, created_at, updated_at
             FROM ai_configs ORDER BY created_at DESC"
        )?;
        
        let configs = stmt.query_map([], |row| {
            Ok(AiConfig {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                api_url: row.get(2)?,
                model_name: row.get(3)?,
                api_key: row.get(4)?,
                temperature: row.get(5)?,
                max_tokens: row.get(6)?,
                is_default: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        
        Ok(configs)
    }
    
    // ========== 动态表管理 ==========
    
    pub fn create_project_data_table(project_id: i64, fields: &[ProjectField]) -> Result<()> {
        let conn = db::get_connection()?;
        
        let field_defs: Vec<(String, String)> = fields
            .iter()
            .map(|f| (f.field_name.clone(), f.field_type.clone()))
            .collect();
        
        db::schema::create_project_table(&conn, project_id, &field_defs)?;
        
        Ok(())
    }
    
    pub fn create_project_dedup_index(project_id: i64, dedup_fields: &[String]) -> Result<()> {
        let conn = db::get_connection()?;
        db::schema::create_dedup_index(&conn, project_id, dedup_fields)?;
        Ok(())
    }
}
