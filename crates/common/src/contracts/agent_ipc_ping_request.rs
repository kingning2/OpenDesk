use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIpcPingRequest {
    pub trace_id: Option<String>,
}
