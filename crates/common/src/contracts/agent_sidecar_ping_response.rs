use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSidecarPingResponse {
    pub ok: bool,
    pub trace_id: Option<String>,
}
