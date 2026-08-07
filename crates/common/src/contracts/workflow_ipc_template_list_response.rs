use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowIpcTemplateListResponse {
    pub templates_json: String,
    pub total: i64,
}
