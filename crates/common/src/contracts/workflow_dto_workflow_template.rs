use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDtoWorkflowTemplate {
    pub id: String,
    pub name: String,
    pub template_type: String,
    pub canvas_json: String,
    pub canvas_version: Option<String>,
    pub canvas_updated: Option<String>,
    pub binding_count: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}
