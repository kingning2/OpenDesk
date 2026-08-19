use common::contracts::{AgentIpcPingRequest, AgentIpcPingResponse};
use common::DingDaResult;

use crate::agent::PingAgent;
use crate::shared::ipc::IpcResponse;
use crate::state::AppState;

#[tauri::command]
pub async fn agent_ping(
    state: tauri::State<'_, AppState>,
    request: AgentIpcPingRequest,
) -> DingDaResult<IpcResponse<AgentIpcPingResponse>> {
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
    let result = PingAgent::execute(state.gateway.as_ref(), request).await?;
    Ok(IpcResponse::ok(result))
}
