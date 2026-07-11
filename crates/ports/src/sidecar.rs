use async_trait::async_trait;
use common::contracts::{AgentSidecarPingRequest, AgentSidecarPingResponse};

#[async_trait]
pub trait AgentSidecarGateway: Send + Sync {
    async fn ping(
        &self,
        request: AgentSidecarPingRequest,
    ) -> Result<AgentSidecarPingResponse, String>;
}
