use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRuntimeEventPhase {
    pub kind: String,
    pub instance_id: String,
    pub node_id: Option<String>,
    pub state: Option<String>,
    pub message: Option<String>,
    pub context_version: Option<i64>,
}
