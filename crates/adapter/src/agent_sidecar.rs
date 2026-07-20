//! HTTP adapter for Python sidecar routes still owned by the runtime layer.

use async_trait::async_trait;
use common::contracts::{AgentSidecarPingRequest, AgentSidecarPingResponse};
use ports::sidecar::AgentSidecarGateway;
use runtime::sidecar::client::SidecarClient;
use runtime::sidecar::routes::agent_ping;

pub struct RuntimeAgentSidecar {
    client: SidecarClient,
}

impl RuntimeAgentSidecar {
    pub fn new(client: SidecarClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl AgentSidecarGateway for RuntimeAgentSidecar {
    async fn ping(
        &self,
        request: AgentSidecarPingRequest,
    ) -> Result<AgentSidecarPingResponse, String> {
        agent_ping::call(&self.client, request)
            .await
            .map_err(|error| error.to_string())
    }
}
