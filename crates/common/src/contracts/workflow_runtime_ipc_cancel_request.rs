use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRuntimeIpcCancelRequest {
    pub instance_id: String,
}
