// 数据库迁移模块

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Statement};

/// 运行所有迁移
pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    tracing::info!("Running database migrations");

    // 创建 projects 表
    create_projects_table(db).await?;

    // 创建 project_fields 表
    create_project_fields_table(db).await?;

    // 创建 ai_configs 表
    create_ai_configs_table(db).await?;

    // 创建 processing_tasks 表
    create_processing_tasks_table(db).await?;

    // 创建 batches 表
    create_batches_table(db).await?;

    // 创建 project_records 表（JSON 统一存储）
    create_project_records_table(db).await?;

    // v0.1.1 迁移：添加 raw_data 列
    add_raw_data_column(db).await?;

    // v0.1.1 迁移：添加 source_files 列到任务表
    add_source_files_column(db).await?;

    // v0.1.2 迁移：创建任务文件进度表
    create_task_file_progress_table(db).await?;

    tracing::info!("Database migrations completed");

    Ok(())
}

async fn create_projects_table(db: &DatabaseConnection) -> Result<(), DbErr> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            dedup_enabled BOOLEAN NOT NULL DEFAULT 1,
            dedup_fields TEXT,
            dedup_strategy TEXT NOT NULL DEFAULT 'skip',
            created_at TEXT NOT NULL,
            updated_at TEXT
        )
    "#;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        sql.to_string(),
    ))
    .await?;

    tracing::debug!("Created projects table");
    Ok(())
}

async fn create_project_fields_table(db: &DatabaseConnection) -> Result<(), DbErr> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS project_fields (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            field_name TEXT NOT NULL,
            field_label TEXT NOT NULL,
            field_type TEXT NOT NULL,
            is_required BOOLEAN NOT NULL DEFAULT 0,
            is_dedup_key BOOLEAN NOT NULL DEFAULT 0,
            is_deleted BOOLEAN NOT NULL DEFAULT 0,
            additional_requirement TEXT,
            validation_rule TEXT,
            extraction_hint TEXT,
            display_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            deleted_at TEXT
        )
    "#;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        sql.to_string(),
    ))
    .await?;

    // 创建索引
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_project_fields_project_id ON project_fields(project_id)".to_string(),
    ))
    .await?;

    tracing::debug!("Created project_fields table");
    Ok(())
}

async fn create_ai_configs_table(db: &DatabaseConnection) -> Result<(), DbErr> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS ai_configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            api_url TEXT NOT NULL,
            model_name TEXT NOT NULL,
            api_key TEXT NOT NULL,
            temperature REAL NOT NULL DEFAULT 0.7,
            max_tokens INTEGER NOT NULL DEFAULT 1000,
            is_default BOOLEAN NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT
        )
    "#;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        sql.to_string(),
    ))
    .await?;

    tracing::debug!("Created ai_configs table");
    Ok(())
}

async fn create_processing_tasks_table(db: &DatabaseConnection) -> Result<(), DbErr> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS processing_tasks (
            id TEXT PRIMARY KEY,
            project_id INTEGER NOT NULL,
            status TEXT NOT NULL,
            total_files INTEGER NOT NULL DEFAULT 0,
            processed_files INTEGER NOT NULL DEFAULT 0,
            total_rows INTEGER NOT NULL DEFAULT 0,
            processed_rows INTEGER NOT NULL DEFAULT 0,
            success_count INTEGER NOT NULL DEFAULT 0,
            error_count INTEGER NOT NULL DEFAULT 0,
            batch_number TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT
        )
    "#;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        sql.to_string(),
    ))
    .await?;

    // 创建索引
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_processing_tasks_project_id ON processing_tasks(project_id)".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_processing_tasks_status ON processing_tasks(status)".to_string(),
    ))
    .await?;

    tracing::debug!("Created processing_tasks table");
    Ok(())
}

async fn create_batches_table(db: &DatabaseConnection) -> Result<(), DbErr> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS batches (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            batch_number TEXT NOT NULL UNIQUE,
            project_id INTEGER NOT NULL,
            file_count INTEGER NOT NULL DEFAULT 0,
            record_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        )
    "#;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        sql.to_string(),
    ))
    .await?;

    // 创建索引
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_batches_project_id ON batches(project_id)".to_string(),
    ))
    .await?;

    tracing::debug!("Created batches table");
    Ok(())
}

async fn create_project_records_table(db: &DatabaseConnection) -> Result<(), DbErr> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS project_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            data TEXT NOT NULL DEFAULT '{}',
            source_file TEXT,
            source_sheet TEXT,
            row_number INTEGER,
            batch_number TEXT,
            status TEXT NOT NULL DEFAULT 'success',
            error_message TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT
        )
    "#;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        sql.to_string(),
    ))
    .await?;

    // 创建索引
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_project_records_project_id ON project_records(project_id)".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_project_records_batch ON project_records(project_id, batch_number)".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_project_records_status ON project_records(project_id, status)".to_string(),
    ))
    .await?;

    tracing::debug!("Created project_records table");
    Ok(())
}

/// v0.1.1 迁移：添加 raw_data 列（原始行数据）
async fn add_raw_data_column(db: &DatabaseConnection) -> Result<(), DbErr> {
    // 检查列是否已存在
    let result = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT name FROM pragma_table_info('project_records') WHERE name = 'raw_data'".to_string(),
        ))
        .await?;

    if result.is_none() {
        db.execute(Statement::from_string(
            db.get_database_backend(),
            "ALTER TABLE project_records ADD COLUMN raw_data TEXT".to_string(),
        ))
        .await?;
        tracing::info!("Added raw_data column to project_records table");
    }

    Ok(())
}

/// v0.1.1 迁移：添加 source_files 列到任务表
async fn add_source_files_column(db: &DatabaseConnection) -> Result<(), DbErr> {
    // 检查列是否已存在
    let result = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT name FROM pragma_table_info('processing_tasks') WHERE name = 'source_files'".to_string(),
        ))
        .await?;

    if result.is_none() {
        db.execute(Statement::from_string(
            db.get_database_backend(),
            "ALTER TABLE processing_tasks ADD COLUMN source_files TEXT".to_string(),
        ))
        .await?;
        tracing::info!("Added source_files column to processing_tasks table");
    }

    Ok(())
}

/// v0.1.2 迁移：创建任务文件进度表
async fn create_task_file_progress_table(db: &DatabaseConnection) -> Result<(), DbErr> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS task_file_progress (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_phase TEXT NOT NULL DEFAULT 'waiting',
            sheet_name TEXT,
            sheet_phase TEXT,
            ai_confidence REAL,
            mapping_count INTEGER,
            success_count INTEGER NOT NULL DEFAULT 0,
            error_count INTEGER NOT NULL DEFAULT 0,
            total_rows INTEGER NOT NULL DEFAULT 0,
            error_message TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT,
            FOREIGN KEY (task_id) REFERENCES processing_tasks(id) ON DELETE CASCADE
        )
    "#;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        sql.to_string(),
    ))
    .await?;

    // 创建索引
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_tfp_task ON task_file_progress(task_id)".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_tfp_file ON task_file_progress(task_id, file_name)".to_string(),
    ))
    .await?;

    tracing::info!("Created task_file_progress table");
    Ok(())
}
