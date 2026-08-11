//! Sidecar route binding: /v1/channel/login (POST)

use common::contracts::{ChannelSidecarLoginRequest, ChannelSidecarLoginResponse};

use crate::sidecar::client::{SidecarClient, SidecarClientError};

pub async fn call(
    client: &SidecarClient,
    request: ChannelSidecarLoginRequest,
) -> Result<ChannelSidecarLoginResponse, SidecarClientError> {
    client.post_json("/v1/channel/login", &request).await
}
