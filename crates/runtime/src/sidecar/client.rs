//! Sidecar HTTP client (Rust → Python).
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-13

use std::time::Instant;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

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

/// 高频探测/轮询路径：正常且够快时只打 DEBUG。
const QUIET_PATHS: &[&str] = &["/health", "/v1/agent/ping", "/v1/channel/qr_check"];
const QUIET_SLOW_MS: u128 = 500;

/// HTTP client for the local Python sidecar. Port is assigned by runtime lifecycle.
///
/// 作者：Xiaoman
/// 创建时间：2026-08-13
#[derive(Clone)]
pub struct SidecarClient {
    base_url: String,
    http: reqwest::Client,
}

impl SidecarClient {
    /// 按端口构造本地 sidecar 客户端。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-13
    ///
    /// # 参数
    /// - `port` — sidecar 监听端口
    ///
    /// # 返回值
    /// 指向 `http://127.0.0.1:{port}` 的客户端。
    pub fn new(port: u16) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            http: reqwest::Client::new(),
        }
    }

    /// 返回 sidecar 根 URL。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-13
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// 探测 `/health`；传输失败视为不健康。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-13
    pub async fn health_check(&self) -> Result<bool, SidecarClientError> {
        match self.get_json::<HealthResponse>("/health").await {
            Ok(response) => Ok(response.status == "ok"),
            Err(SidecarClientError::Transport(_)) => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// GET JSON。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-13
    ///
    /// # 参数
    /// - `path` — 相对路径（如 `/health`）
    ///
    /// # 返回值
    /// 反序列化后的响应体。
    pub async fn get_json<Res>(&self, path: &str) -> Result<Res, SidecarClientError>
    where
        Res: DeserializeOwned,
    {
        let started = Instant::now();
        let url = format!("{}{}", self.base_url, path);
        let result = async {
            let response = self
                .http
                .get(&url)
                .send()
                .await
                .map_err(|error| SidecarClientError::Transport(error.to_string()))?;

            let status = response.status();
            if !status.is_success() {
                return Err(SidecarClientError::Sidecar(format!(
                    "unexpected status {status}"
                )));
            }

            response
                .json()
                .await
                .map_err(|error| SidecarClientError::Transport(error.to_string()))
        }
        .await;

        log_http_result("GET", path, started, &result);
        result
    }

    /// GET 纯文本。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-13
    ///
    /// # 参数
    /// - `path` — 相对路径
    ///
    /// # 返回值
    /// 响应文本。
    pub async fn get_text(&self, path: &str) -> Result<String, SidecarClientError> {
        let started = Instant::now();
        let url = format!("{}{}", self.base_url, path);
        let result = async {
            let response = self
                .http
                .get(&url)
                .send()
                .await
                .map_err(|error| SidecarClientError::Transport(error.to_string()))?;

            let status = response.status();
            if !status.is_success() {
                return Err(SidecarClientError::Sidecar(format!(
                    "unexpected status {status}"
                )));
            }

            response
                .text()
                .await
                .map_err(|error| SidecarClientError::Transport(error.to_string()))
        }
        .await;

        log_http_result("GET", path, started, &result);
        result
    }

    /// POST JSON。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-13
    ///
    /// # 参数
    /// - `path` — 相对路径（如 `/v1/llm/chat`）
    /// - `body` — 可序列化请求体
    ///
    /// # 返回值
    /// 反序列化后的响应体。
    pub async fn post_json<Req, Res>(
        &self,
        path: &str,
        body: &Req,
    ) -> Result<Res, SidecarClientError>
    where
        Req: Serialize + ?Sized,
        Res: DeserializeOwned,
    {
        let started = Instant::now();
        let url = format!("{}{}", self.base_url, path);
        let result = async {
            let response = self
                .http
                .post(&url)
                .json(body)
                .send()
                .await
                .map_err(|error| SidecarClientError::Transport(error.to_string()))?;

            let status = response.status();
            if !status.is_success() {
                return Err(SidecarClientError::Sidecar(format!(
                    "unexpected status {status}"
                )));
            }

            response
                .json()
                .await
                .map_err(|error| SidecarClientError::Transport(error.to_string()))
        }
        .await;

        log_http_result("POST", path, started, &result);
        result
    }
}

fn log_http_result<T>(
    method: &str,
    path: &str,
    started: Instant,
    result: &Result<T, SidecarClientError>,
) {
    let duration_ms = started.elapsed().as_millis();
    match result {
        Ok(_) => {
            let quiet = QUIET_PATHS.contains(&path) && duration_ms < QUIET_SLOW_MS;
            if quiet {
                debug!(method, path, duration_ms, "sidecar HTTP 调用完成");
            } else {
                info!(method, path, duration_ms, "sidecar HTTP 调用完成");
            }
        }
        Err(error) => {
            warn!(
                method,
                path,
                duration_ms,
                error = %error,
                "sidecar HTTP 调用失败"
            );
        }
    }
}
