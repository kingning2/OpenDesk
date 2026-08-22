//! Sidecar route binding: /v1/llm/classify (POST)

use common::contracts::{LlmIpcClassifyRequest, LlmIpcClassifyResponse};

use crate::sidecar::client::{SidecarClient, SidecarClientError};

pub async fn call(
    client: &SidecarClient,
    request: LlmIpcClassifyRequest,
) -> Result<LlmIpcClassifyResponse, SidecarClientError> {
    client.post_json("/v1/llm/classify", &request).await
}
