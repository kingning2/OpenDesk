//! Sidecar route binding: /v1/crawler/job/status (POST)

use common::contracts::{CrawlerSidecarJobStatusRequest, CrawlerSidecarJobStatusResponse};

use crate::sidecar::client::{SidecarClient, SidecarClientError};

pub async fn call(
    client: &SidecarClient,
    request: CrawlerSidecarJobStatusRequest,
) -> Result<CrawlerSidecarJobStatusResponse, SidecarClientError> {
    client.post_json("/v1/crawler/job/status", &request).await
}
