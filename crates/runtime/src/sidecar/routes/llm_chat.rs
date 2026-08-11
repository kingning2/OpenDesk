//! Sidecar route binding: /v1/llm/chat (POST)

use common::contracts::{LlmIpcChatRequest, LlmIpcChatResponse};

use crate::sidecar::client::{SidecarClient, SidecarClientError};

pub async fn call(
    client: &SidecarClient,
    request: LlmIpcChatRequest,
) -> Result<LlmIpcChatResponse, SidecarClientError> {
    client.post_json("/v1/llm/chat", &request).await
}
