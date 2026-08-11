//! Agent 业务层：与 sidecar 的 ping 交互。
//!
//! 作者：coisini
//! 创建时间：2026-07-21

use common::contracts::{AgentIpcPingRequest, AgentIpcPingResponse, AgentSidecarPingRequest};
use ports::sidecar::AgentSidecarGateway;

/// PingAgent：把 IPC ping 请求转发给 sidecar 网关。
///
/// 作者：coisini
/// 创建时间：2026-07-21
pub struct PingAgent;

impl PingAgent {
    /// 执行一次 agent ping。
    ///
    /// # 参数
    /// - `gateway` — sidecar 网关
    /// - `request` — IPC ping 请求
    ///
    /// # 返回值
    /// 网关 ping 响应。
    pub async fn execute<G: AgentSidecarGateway + ?Sized>(
        gateway: &G,
        request: AgentIpcPingRequest,
    ) -> Result<AgentIpcPingResponse, String> {
        let sidecar_request = AgentSidecarPingRequest {
            trace_id: request.trace_id,
        };
        let sidecar_response = gateway.ping(sidecar_request).await?;
        Ok(AgentIpcPingResponse {
            ok: sidecar_response.ok,
            trace_id: sidecar_response.trace_id,
        })
    }
}
