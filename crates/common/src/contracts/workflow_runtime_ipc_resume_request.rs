use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRuntimeIpcResumeRequest {
    pub instance_id: String,
}
