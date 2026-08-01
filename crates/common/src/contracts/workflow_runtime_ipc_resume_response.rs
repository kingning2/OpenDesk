use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRuntimeIpcResumeResponse {
    pub instance_id: String,
    pub state: String,
    pub error: Option<String>,
}
