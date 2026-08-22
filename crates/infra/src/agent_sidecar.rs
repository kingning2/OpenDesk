//! HTTP adapter for Python sidecar routes still owned by the runtime layer.

use crate::sidecar::client::SidecarClient;
use crate::sidecar::routes::agent_ping;
use async_trait::async_trait;
use common::contracts::{AgentSidecarPingRequest, AgentSidecarPingResponse};
use common::DingDaResult;
use ports::sidecar::AgentSidecarGateway;

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
    ) -> DingDaResult<AgentSidecarPingResponse> {
        agent_ping::call(&self.client, request)
            .await
            .map_err(|error| common::DingDaError::Internal(error.to_string()))
    }
}
