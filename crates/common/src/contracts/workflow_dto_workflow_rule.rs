use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDtoWorkflowRule {
    pub id: String,
    pub name: String,
    pub from_stages_json: String,
    pub to_stage: String,
    pub trigger_keywords_json: String,
    pub trigger_tags_json: String,
    pub auto_reply: bool,
    pub auto_advance: bool,
    pub reply_script_id: Option<String>,
}
