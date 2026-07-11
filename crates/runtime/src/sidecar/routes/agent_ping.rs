//! Sidecar route binding: /v1/agent/ping (POST)

use crate::sidecar::client::{SidecarClient, SidecarClientError};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AgentPingRequest {
    pub trace_id: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AgentPingResponse {
    pub ok: bool,
    pub trace_id: String,
}

pub async fn agent_ping(
    client: &SidecarClient,
    request: AgentPingRequest,
) -> Result<AgentPingResponse, SidecarClientError> {
    let _ = (client, request);
    // TODO: POST /v1/agent/ping with contract DTO; parse response JSON.
    Err(SidecarClientError::Transport(
        "not implemented (skeleton)".into(),
    ))
}
