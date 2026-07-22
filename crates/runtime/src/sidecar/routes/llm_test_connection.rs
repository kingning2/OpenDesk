//! Sidecar route: POST /v1/runtime/llm_test_connection
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-22

use common::contracts::{
    RuntimeSidecarLlmTestConnectionRequest, RuntimeSidecarLlmTestConnectionResponse,
};

use crate::sidecar::client::{SidecarClient, SidecarClientError};

/// 调用 Python Sidecar 做 LLM 连通性探测。
///
/// # 参数
/// - `client` — Sidecar HTTP 客户端
/// - `request` — 含内存态 api_key（不落盘）
///
/// # 返回值
/// Sidecar 探测结果。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub async fn call(
    client: &SidecarClient,
    request: RuntimeSidecarLlmTestConnectionRequest,
) -> Result<RuntimeSidecarLlmTestConnectionResponse, SidecarClientError> {
    client
        .post_json("/v1/runtime/llm_test_connection", &request)
        .await
}
