//! Agent ?? Tauri commands?
//!
//! ???coisini
//! ?????2026-07-21

use common::contracts::{AgentIpcPingRequest, AgentIpcPingResponse};
use opendesk_macros::timed;

use crate::agent::PingAgent;
use crate::state::AppState;

/// Agent ping IPC????????????????
///
/// ???coisini
/// ?????2026-07-16
///
/// # ??
/// - `state` ? ??????
/// - `request` ? ping ???? trace?
///
/// # ???
/// Sidecar ping ??????? sidecar ????????????
#[tauri::command]
#[timed]
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
