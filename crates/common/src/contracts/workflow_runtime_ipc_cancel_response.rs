use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRuntimeIpcCancelResponse {
    pub instance_id: String,
    pub ok: bool,
    pub error: Option<String>,
}
