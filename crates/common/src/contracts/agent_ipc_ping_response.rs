use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIpcPingResponse {
    pub ok: bool,
    pub trace_id: Option<String>,
}
