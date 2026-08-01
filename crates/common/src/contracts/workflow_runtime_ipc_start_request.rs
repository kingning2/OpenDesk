use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRuntimeIpcStartRequest {
    pub definition_json: String,
    pub context_json: Option<String>,
}
