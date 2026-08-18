use async_trait::async_trait;
use common::contracts::{AgentSidecarPingRequest, AgentSidecarPingResponse};
use common::OpenDeskResult;

/// Sidecar gateway for runtime features (e.g. agent ping).
#[async_trait]
pub trait AgentSidecarGateway: Send + Sync {
    async fn ping(
        &self,
        request: AgentSidecarPingRequest,
    ) -> OpenDeskResult<AgentSidecarPingResponse>;
}
