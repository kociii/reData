use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub dedup_enabled: bool,
    pub dedup_fields: Option<Vec<String>>,
    pub dedup_strategy: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Project {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            id: None,
            name,
            description,
            dedup_enabled: true,
            dedup_fields: None,
            dedup_strategy: "skip".to_string(),
            created_at: None,
            updated_at: None,
        }
    }
}
