//! Sidecar HTTP client (Rust → Python). Skeleton only.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SidecarClientError {
    #[error("transport: {0}")]
    Transport(String),
    #[error("sidecar: {0}")]
    Sidecar(String),
}

/// HTTP client for the local Python sidecar. Port is assigned by runtime lifecycle.
pub struct SidecarClient {
    base_url: String,
}

impl SidecarClient {
    pub fn new(port: u16) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    // TODO: inject reqwest::Client when runtime wiring is approved.
}
