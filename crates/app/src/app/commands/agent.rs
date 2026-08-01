//! Agent 相关 Tauri commands。
//!
//! 作者：coisini
//! 创建时间：2026-07-21

use common::contracts::{AgentIpcPingRequest, AgentIpcPingResponse};

use crate::app::commands::llm::stored_llm_client;
use crate::app::state::AppState;

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
/// 保持原有 `ok + trace_id` 响应；未授权时返回错误字符串。
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
    let ok = match stored_llm_client(&state).await {
        Ok(client) => client.test_connection().await.is_ok(),
        Err(_) => false,
    };
    Ok(AgentIpcPingResponse {
        ok,
        trace_id: request.trace_id,
    })
}
