use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDtoWorkflowScript {
    pub id: String,
    pub stage: Option<String>,
    pub category_l1: Option<String>,
    pub category_l2: Option<String>,
    pub trigger_text: Option<String>,
    pub description: Option<String>,
    pub from_stage: Option<String>,
    pub to_stage: Option<String>,
    pub tags_json: String,
    pub content: String,
    pub needs_boss_input: bool,
    pub boss_input_hint: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}
