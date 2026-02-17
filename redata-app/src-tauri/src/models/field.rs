use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectField {
    pub id: Option<i64>,
    pub project_id: i64,
    pub field_name: String,
    pub field_label: String,
    pub field_type: String,
    pub is_required: bool,
    pub validation_rule: Option<String>,
    pub extraction_hint: Option<String>,
    pub display_order: i32,
    pub created_at: Option<String>,
}

impl ProjectField {
    pub fn new(
        project_id: i64,
        field_name: String,
        field_label: String,
        field_type: String,
    ) -> Self {
        Self {
            id: None,
            project_id,
            field_name,
            field_label,
            field_type,
            is_required: false,
            validation_rule: None,
            extraction_hint: None,
            display_order: 0,
            created_at: None,
        }
    }
}
