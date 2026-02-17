use rusqlite::{Connection, Result};

pub fn create_tables(conn: &Connection) -> Result<()> {
    // 1. 项目表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            dedup_enabled INTEGER DEFAULT 1,
            dedup_fields TEXT,
            dedup_strategy TEXT DEFAULT 'skip',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // 2. 项目字段定义表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS project_fields (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            field_name TEXT NOT NULL,
            field_label TEXT NOT NULL,
            field_type TEXT NOT NULL,
            is_required INTEGER DEFAULT 0,
            validation_rule TEXT,
            extraction_hint TEXT,
            display_order INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_project_fields_project_id 
         ON project_fields(project_id)",
        [],
    )?;

    // 3. 处理任务表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS processing_tasks (
            id TEXT PRIMARY KEY,
            project_id INTEGER NOT NULL,
            status TEXT NOT NULL,
            total_files INTEGER DEFAULT 0,
            processed_files INTEGER DEFAULT 0,
            total_rows INTEGER DEFAULT 0,
            processed_rows INTEGER DEFAULT 0,
            success_count INTEGER DEFAULT 0,
            error_count INTEGER DEFAULT 0,
            batch_number TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_task_status 
         ON processing_tasks(status)",
        [],
    )?;

    // 4. AI 配置表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            api_url TEXT NOT NULL,
            model_name TEXT NOT NULL,
            api_key TEXT NOT NULL,
            temperature REAL DEFAULT 0.7,
            max_tokens INTEGER DEFAULT 1000,
            is_default INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // 5. 批次表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS batches (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            batch_number TEXT NOT NULL UNIQUE,
            project_id INTEGER NOT NULL,
            file_count INTEGER DEFAULT 0,
            record_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(())
}

/// 为项目创建动态数据表
pub fn create_project_table(conn: &Connection, project_id: i64, fields: &[(String, String)]) -> Result<()> {
    let table_name = format!("project_{}_records", project_id);
    
    // 构建字段定义
    let mut field_defs = Vec::new();
    for (field_name, field_type) in fields {
        let sql_type = match field_type.as_str() {
            "number" => "REAL",
            _ => "TEXT",
        };
        field_defs.push(format!("{} {}", field_name, sql_type));
    }
    
    // 固定字段
    let fixed_fields = vec![
        "id INTEGER PRIMARY KEY AUTOINCREMENT",
        "raw_content TEXT",
        "source_file TEXT",
        "source_sheet TEXT",
        "row_number INTEGER",
        "batch_number TEXT",
        "status TEXT DEFAULT 'success'",
        "error_message TEXT",
        "created_at DATETIME DEFAULT CURRENT_TIMESTAMP",
        "updated_at DATETIME DEFAULT CURRENT_TIMESTAMP",
    ];
    
    let all_fields = [field_defs.join(", "), fixed_fields.join(", ")].join(", ");
    
    let create_sql = format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        table_name, all_fields
    );
    
    conn.execute(&create_sql, [])?;
    
    Ok(())
}

/// 为项目数据表创建去重索引
pub fn create_dedup_index(
    conn: &Connection,
    project_id: i64,
    dedup_fields: &[String],
) -> Result<()> {
    let table_name = format!("project_{}_records", project_id);
    let index_name = format!("idx_dedup_{}", project_id);
    let fields_str = dedup_fields.join(", ");
    
    let create_index_sql = format!(
        "CREATE UNIQUE INDEX IF NOT EXISTS {} ON {} ({})",
        index_name, table_name, fields_str
    );
    
    conn.execute(&create_index_sql, [])?;
    
    Ok(())
}
