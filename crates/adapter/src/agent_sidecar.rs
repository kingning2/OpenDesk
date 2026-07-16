use async_trait::async_trait;
use common::contracts::{
    AgentSidecarPingRequest, AgentSidecarPingResponse, CrawlerSidecarJobCancelRequest,
    CrawlerSidecarJobCancelResponse, CrawlerSidecarJobLogsRequest, CrawlerSidecarJobLogsResponse,
    CrawlerSidecarJobStartRequest, CrawlerSidecarJobStartResponse, CrawlerSidecarJobStatusRequest,
    CrawlerSidecarJobStatusResponse,
};
use ports::sidecar::{AgentSidecarGateway, CrawlerSidecarGateway};
use runtime::sidecar::client::SidecarClient;
use runtime::sidecar::routes::{
    agent_ping, crawler_job_cancel, crawler_job_logs, crawler_job_start, crawler_job_status,
};

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

#[async_trait]
impl CrawlerSidecarGateway for RuntimeAgentSidecar {
    async fn job_start(
        &self,
        request: CrawlerSidecarJobStartRequest,
    ) -> Result<CrawlerSidecarJobStartResponse, String> {
        crawler_job_start::call(&self.client, request)
            .await
            .map_err(|error| error.to_string())
    }

    async fn job_cancel(
        &self,
        request: CrawlerSidecarJobCancelRequest,
    ) -> Result<CrawlerSidecarJobCancelResponse, String> {
        crawler_job_cancel::call(&self.client, request)
            .await
            .map_err(|error| error.to_string())
    }

    async fn job_status(
        &self,
        request: CrawlerSidecarJobStatusRequest,
    ) -> Result<CrawlerSidecarJobStatusResponse, String> {
        crawler_job_status::call(&self.client, request)
            .await
            .map_err(|error| error.to_string())
    }

    async fn job_logs(
        &self,
        request: CrawlerSidecarJobLogsRequest,
    ) -> Result<CrawlerSidecarJobLogsResponse, String> {
        crawler_job_logs::call(&self.client, request)
            .await
            .map_err(|error| error.to_string())
    }
}
