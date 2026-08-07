use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowIpcRuleListResponse {
    pub rules_json: String,
    pub total: i64,
}
