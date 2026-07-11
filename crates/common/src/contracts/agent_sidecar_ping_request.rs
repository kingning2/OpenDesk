use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSidecarPingRequest {
    pub trace_id: Option<String>,
}
