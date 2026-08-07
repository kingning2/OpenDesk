use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowIpcScriptListResponse {
    pub scripts_json: String,
    pub total: i64,
}
