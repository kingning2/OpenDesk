//! Sidecar management API bindings (Rust → Python).

use crate::sidecar::client::{SidecarClient, SidecarClientError};

pub async fn stats(client: &SidecarClient) -> Result<serde_json::Value, SidecarClientError> {
    client.get_json("/stats").await
}

pub async fn active_tasks(client: &SidecarClient) -> Result<serde_json::Value, SidecarClientError> {
    client.get_json("/tasks/active").await
}

pub async fn debug_dump(client: &SidecarClient) -> Result<serde_json::Value, SidecarClientError> {
    client.get_json("/debug/dump").await
}

pub async fn metrics(client: &SidecarClient) -> Result<String, SidecarClientError> {
    client.get_text("/metrics").await
}
