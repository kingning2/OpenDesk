use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDtoWorkflowStage {
    pub template_id: String,
    pub id: String,
    pub name: String,
    pub note: Option<String>,
    pub ord: i64,
    pub ai_level: Option<String>,
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub scripts_json: String,
    pub script_conds_json: String,
}
