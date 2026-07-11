use common::contracts::{AgentIpcPingRequest, AgentIpcPingResponse, AgentSidecarPingRequest};
use ports::sidecar::AgentSidecarGateway;

pub struct PingAgent;

impl PingAgent {
    pub async fn execute<G: AgentSidecarGateway + ?Sized>(
        gateway: &G,
        request: AgentIpcPingRequest,
    ) -> Result<AgentIpcPingResponse, String> {
        let sidecar_request = AgentSidecarPingRequest {
            trace_id: request.trace_id,
        };
        let sidecar_response = gateway.ping(sidecar_request).await?;
        Ok(AgentIpcPingResponse {
            ok: sidecar_response.ok,
            trace_id: sidecar_response.trace_id,
        })
    }
}
