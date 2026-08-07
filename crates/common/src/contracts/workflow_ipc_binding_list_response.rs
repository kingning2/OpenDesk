use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowIpcBindingListResponse {
    pub bindings_json: String,
    pub total: i64,
}
