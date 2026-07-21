//! Agent 相关 Tauri commands。
//!
//! 作者：coisini
//! 创建时间：2026-07-21

use agent::app::ping::PingAgent;
use common::contracts::{AgentIpcPingRequest, AgentIpcPingResponse};

use crate::state::AppState;

/// Agent ping IPC；有锁构建下会先执行授权硬检查。
///
/// 作者：coisini
/// 创建时间：2026-07-16
///
/// # 参数
/// - `state` — 应用共享状态
/// - `request` — ping 请求（含 trace）
///
/// # 返回值
/// Sidecar ping 响应；未授权或 sidecar 不可用时返回错误字符串。
#[tauri::command]
pub async fn agent_ping(
    state: tauri::State<'_, AppState>,
    request: AgentIpcPingRequest,
) -> Result<AgentIpcPingResponse, String> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(|error| error.to_string())?;
    state
        .lifecycle
        .ensure_running()
        .await
        .map_err(|error| error.to_string())?;
    PingAgent::execute(state.gateway.as_ref(), request).await
}
