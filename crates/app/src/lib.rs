mod logging;
mod state;

use adapter::agent_sidecar::RuntimeAgentSidecar;
use agent::app::ping::PingAgent;
use common::contracts::{AgentIpcPingRequest, AgentIpcPingResponse};
use common::license::{LicenseActivateRequest, LicenseStatus};
use kernel::event::{EventBus, InMemoryEventBus};
use logging::init_tracing;
use runtime::sidecar::lifecycle::{SidecarConfig, SidecarLifecycle};
use state::{build_license_gate, AppState};
use std::sync::Arc;
use tauri::Manager;

/// Agent ping IPC；有锁构建下会先执行授权硬检查。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
///
/// # 参数
///
/// * `state` - 应用共享状态
/// * `request` - ping 请求体
///
/// # 返回值
///
/// 成功返回 ping 响应；未授权或 sidecar 失败时返回错误字符串。
#[tauri::command]
async fn agent_ping(
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

/// 查询授权状态 IPC。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
///
/// # 参数
///
/// * `state` - 应用共享状态
///
/// # 返回值
///
/// 成功返回 [`LicenseStatus`]；失败返回错误字符串。
#[tauri::command]
async fn license_status(state: tauri::State<'_, AppState>) -> Result<LicenseStatus, String> {
    state
        .license
        .status()
        .await
        .map_err(|error| error.to_string())
}

/// 读取本机设备码 IPC。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
///
/// # 参数
///
/// * `state` - 应用共享状态
///
/// # 返回值
///
/// 成功返回设备码；失败返回错误字符串。
#[tauri::command]
async fn license_machine_code(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state
        .license
        .machine_code()
        .await
        .map_err(|error| error.to_string())
}

/// 激活授权 IPC。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
///
/// # 参数
///
/// * `state` - 应用共享状态
/// * `request` - token 或 key Base64 激活请求
///
/// # 返回值
///
/// 成功返回激活后的 [`LicenseStatus`]；失败返回错误字符串。
#[tauri::command]
async fn license_activate(
    state: tauri::State<'_, AppState>,
    request: LicenseActivateRequest,
) -> Result<LicenseStatus, String> {
    state
        .license
        .activate(request)
        .await
        .map_err(|error| error.to_string())
}

pub fn launch(context: tauri::Context<tauri::Wry>) -> tauri::Result<()> {
    init_tracing();

    let event_bus = Arc::new(InMemoryEventBus::new());
    let lifecycle = Arc::new(SidecarLifecycle::new(
        SidecarConfig::from_env(),
        event_bus.clone() as Arc<dyn EventBus>,
    ));
    let gateway = Arc::new(RuntimeAgentSidecar::new(lifecycle.client().clone()));
    let license = build_license_gate();
    let app_state = AppState {
        lifecycle: lifecycle.clone(),
        gateway,
        license,
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
        .invoke_handler(tauri::generate_handler![
            agent_ping,
            license_status,
            license_machine_code,
            license_activate
        ])
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
