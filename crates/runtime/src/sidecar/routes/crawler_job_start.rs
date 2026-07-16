//! Sidecar route binding: /v1/crawler/job/start (POST)

use common::contracts::{CrawlerSidecarJobStartRequest, CrawlerSidecarJobStartResponse};

use crate::sidecar::client::{SidecarClient, SidecarClientError};

pub async fn call(
    client: &SidecarClient,
    request: CrawlerSidecarJobStartRequest,
) -> Result<CrawlerSidecarJobStartResponse, SidecarClientError> {
    client.post_json("/v1/crawler/job/start", &request).await
}
