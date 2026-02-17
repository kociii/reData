use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingTask {
    pub id: String,
    pub project_id: i64,
    pub status: String,
    pub total_files: i32,
    pub processed_files: i32,
    pub total_rows: i32,
    pub processed_rows: i32,
    pub success_count: i32,
    pub error_count: i32,
    pub batch_number: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl ProcessingTask {
    pub fn new(project_id: i64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id,
            status: "pending".to_string(),
            total_files: 0,
            processed_files: 0,
            total_rows: 0,
            processed_rows: 0,
            success_count: 0,
            error_count: 0,
            batch_number: None,
            created_at: None,
            updated_at: None,
        }
    }
}
