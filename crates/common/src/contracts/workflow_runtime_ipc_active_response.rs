use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRuntimeIpcActiveResponse {
    pub instances_json: String,
}
