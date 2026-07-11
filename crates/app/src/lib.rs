mod logging;
mod state;

use adapter::agent_sidecar::RuntimeAgentSidecar;
use agent::app::ping::PingAgent;
use common::contracts::{AgentIpcPingRequest, AgentIpcPingResponse};
use kernel::event::{EventBus, InMemoryEventBus};
use logging::init_tracing;
use runtime::sidecar::lifecycle::{SidecarConfig, SidecarLifecycle};
use state::AppState;
use std::sync::Arc;
use tauri::Manager;

#[tauri::command]
async fn agent_ping(
    state: tauri::State<'_, AppState>,
    request: AgentIpcPingRequest,
) -> Result<AgentIpcPingResponse, String> {
    state
        .lifecycle
        .ensure_running()
        .await
        .map_err(|error| error.to_string())?;
    PingAgent::execute(state.gateway.as_ref(), request).await
}

pub fn launch(context: tauri::Context<tauri::Wry>) -> tauri::Result<()> {
    init_tracing();

    let event_bus = Arc::new(InMemoryEventBus::new());
    let lifecycle = Arc::new(SidecarLifecycle::new(
        SidecarConfig::from_env(),
        event_bus.clone() as Arc<dyn EventBus>,
    ));
    let gateway = Arc::new(RuntimeAgentSidecar::new(lifecycle.client().clone()));
    let app_state = AppState {
        lifecycle: lifecycle.clone(),
        gateway,
        event_bus,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .setup(move |app| {
            let lifecycle = app.state::<AppState>().lifecycle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = lifecycle.ensure_running().await {
                    tracing::error!(%error, "sidecar startup failed");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![agent_ping])
        .build(context)?
        .run(move |app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                let lifecycle = app_handle.state::<AppState>().lifecycle.clone();
                tauri::async_runtime::block_on(async move {
                    if let Err(error) = lifecycle.stop().await {
                        tracing::error!(%error, "sidecar shutdown failed");
                    }
                });
            }
        });

    Ok(())
}
