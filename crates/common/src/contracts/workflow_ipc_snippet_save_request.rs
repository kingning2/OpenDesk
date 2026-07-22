use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowIpcSnippetSaveRequest {
    pub id: Option<String>,
    pub title: String,
    pub stage: Option<String>,
    pub trigger_text: Option<String>,
    pub description: Option<String>,
    pub from_stage: Option<String>,
    pub to_stage: Option<String>,
    pub tags_json: String,
    pub body_text: String,
    pub category_l1: Option<String>,
    pub category_l2: Option<String>,
    pub needs_boss_input: bool,
    pub boss_input_hint: Option<String>,
    pub sort_order: i64,
}
