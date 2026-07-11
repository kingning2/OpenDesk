//! Sidecar HTTP client (Rust → Python).

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SidecarClientError {
    #[error("transport: {0}")]
    Transport(String),
    #[error("sidecar: {0}")]
    Sidecar(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
}

/// HTTP client for the local Python sidecar. Port is assigned by runtime lifecycle.
#[derive(Clone)]
pub struct SidecarClient {
    base_url: String,
    http: reqwest::Client,
}

impl SidecarClient {
    pub fn new(port: u16) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            http: reqwest::Client::new(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn health_check(&self) -> Result<bool, SidecarClientError> {
        match self.get_json::<HealthResponse>("/health").await {
            Ok(response) => Ok(response.status == "ok"),
            Err(SidecarClientError::Transport(_)) => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub async fn get_json<Res>(&self, path: &str) -> Result<Res, SidecarClientError>
    where
        Res: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|error| SidecarClientError::Transport(error.to_string()))?;

        if !response.status().is_success() {
            return Err(SidecarClientError::Sidecar(format!(
                "unexpected status {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|error| SidecarClientError::Transport(error.to_string()))
    }

    pub async fn get_text(&self, path: &str) -> Result<String, SidecarClientError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|error| SidecarClientError::Transport(error.to_string()))?;

        if !response.status().is_success() {
            return Err(SidecarClientError::Sidecar(format!(
                "unexpected status {}",
                response.status()
            )));
        }

        response
            .text()
            .await
            .map_err(|error| SidecarClientError::Transport(error.to_string()))
    }

    pub async fn post_json<Req, Res>(
        &self,
        path: &str,
        body: &Req,
    ) -> Result<Res, SidecarClientError>
    where
        Req: Serialize + ?Sized,
        Res: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .post(url)
            .json(body)
            .send()
            .await
            .map_err(|error| SidecarClientError::Transport(error.to_string()))?;

        if !response.status().is_success() {
            return Err(SidecarClientError::Sidecar(format!(
                "unexpected status {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|error| SidecarClientError::Transport(error.to_string()))
    }
}
