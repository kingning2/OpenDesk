//! Sidecar route binding: /v1/crawler/job/logs (POST)

use common::contracts::{CrawlerSidecarJobLogsRequest, CrawlerSidecarJobLogsResponse};

use crate::sidecar::client::{SidecarClient, SidecarClientError};

pub async fn call(
    client: &SidecarClient,
    request: CrawlerSidecarJobLogsRequest,
) -> Result<CrawlerSidecarJobLogsResponse, SidecarClientError> {
    client.post_json("/v1/crawler/job/logs", &request).await
}
