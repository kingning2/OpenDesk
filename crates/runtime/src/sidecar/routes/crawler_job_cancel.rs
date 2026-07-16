//! Sidecar route binding: /v1/crawler/job/cancel (POST)

use common::contracts::{CrawlerSidecarJobCancelRequest, CrawlerSidecarJobCancelResponse};

use crate::sidecar::client::{SidecarClient, SidecarClientError};

pub async fn call(
    client: &SidecarClient,
    request: CrawlerSidecarJobCancelRequest,
) -> Result<CrawlerSidecarJobCancelResponse, SidecarClientError> {
    client.post_json("/v1/crawler/job/cancel", &request).await
}
