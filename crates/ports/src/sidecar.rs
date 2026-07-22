use async_trait::async_trait;
use common::contracts::{
    AgentSidecarPingRequest, AgentSidecarPingResponse, RuntimeSidecarLlmTestConnectionRequest,
    RuntimeSidecarLlmTestConnectionResponse,
};

/// Sidecar gateway for non-crawler runtime features (e.g. agent ping, LLM probe).
#[async_trait]
pub trait AgentSidecarGateway: Send + Sync {
    async fn ping(
        &self,
        request: AgentSidecarPingRequest,
    ) -> Result<AgentSidecarPingResponse, String>;

    /// 转发 LLM 连通性探测（密钥仅内存传递）。
    ///
    /// # 参数
    /// - `request` — Sidecar 探测请求
    ///
    /// # 返回值
    /// 探测结果；失败时为错误字符串。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-22
    async fn llm_test_connection(
        &self,
        request: RuntimeSidecarLlmTestConnectionRequest,
    ) -> Result<RuntimeSidecarLlmTestConnectionResponse, String>;
}
