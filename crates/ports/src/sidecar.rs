use async_trait::async_trait;
use common::contracts::{
    AgentSidecarPingRequest, AgentSidecarPingResponse, CrawlerSidecarJobCancelRequest,
    CrawlerSidecarJobCancelResponse, CrawlerSidecarJobLogsRequest, CrawlerSidecarJobLogsResponse,
    CrawlerSidecarJobStartRequest, CrawlerSidecarJobStartResponse, CrawlerSidecarJobStatusRequest,
    CrawlerSidecarJobStatusResponse,
};

#[async_trait]
pub trait AgentSidecarGateway: Send + Sync {
    async fn ping(
        &self,
        request: AgentSidecarPingRequest,
    ) -> Result<AgentSidecarPingResponse, String>;
}

#[async_trait]
pub trait CrawlerSidecarGateway: Send + Sync {
    async fn job_start(
        &self,
        request: CrawlerSidecarJobStartRequest,
    ) -> Result<CrawlerSidecarJobStartResponse, String>;

    async fn job_cancel(
        &self,
        request: CrawlerSidecarJobCancelRequest,
    ) -> Result<CrawlerSidecarJobCancelResponse, String>;

    async fn job_status(
        &self,
        request: CrawlerSidecarJobStatusRequest,
    ) -> Result<CrawlerSidecarJobStatusResponse, String>;

    async fn job_logs(
        &self,
        request: CrawlerSidecarJobLogsRequest,
    ) -> Result<CrawlerSidecarJobLogsResponse, String>;
}
