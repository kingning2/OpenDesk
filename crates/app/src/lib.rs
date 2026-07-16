mod logging;
mod state;

use adapter::agent_sidecar::RuntimeAgentSidecar;
use agent::app::ping::PingAgent;
use common::contracts::{
    AgentIpcPingRequest, AgentIpcPingResponse, CrawlerIpcJobCancelRequest,
    CrawlerIpcJobCancelResponse, CrawlerIpcJobLogsRequest, CrawlerIpcJobLogsResponse,
    CrawlerIpcJobStartRequest, CrawlerIpcJobStartResponse, CrawlerIpcJobStatusRequest,
    CrawlerIpcJobStatusResponse, CrawlerSidecarJobCancelRequest, CrawlerSidecarJobLogsRequest,
    CrawlerSidecarJobStartRequest, CrawlerSidecarJobStatusRequest,
};
use kernel::event::{EventBus, InMemoryEventBus};
use logging::init_tracing;
use ports::sidecar::CrawlerSidecarGateway;
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

#[tauri::command]
async fn crawler_job_start(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcJobStartRequest,
) -> Result<CrawlerIpcJobStartResponse, String> {
    state
        .lifecycle
        .ensure_running()
        .await
        .map_err(|error| error.to_string())?;
    let sidecar_request = CrawlerSidecarJobStartRequest {
        trace_id: request.trace_id.clone(),
        platform: request.platform,
        keywords: request.keywords,
        rate_limit_ms: request.rate_limit_ms,
        max_total: request.max_total,
        year: request.year,
        min_year_video_count: request.min_year_video_count,
        exclude_countries: request.exclude_countries,
        batch_id: request.batch_id,
        api_key: request.api_key,
    };
    let response = state.gateway.job_start(sidecar_request).await?;
    Ok(CrawlerIpcJobStartResponse {
        ok: response.ok,
        job_id: response.job_id,
        trace_id: response.trace_id.or(request.trace_id),
    })
}

#[tauri::command]
async fn crawler_job_cancel(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcJobCancelRequest,
) -> Result<CrawlerIpcJobCancelResponse, String> {
    state
        .lifecycle
        .ensure_running()
        .await
        .map_err(|error| error.to_string())?;
    let sidecar_request = CrawlerSidecarJobCancelRequest {
        trace_id: request.trace_id.clone(),
        job_id: request.job_id.clone(),
    };
    let response = state.gateway.job_cancel(sidecar_request).await?;
    Ok(CrawlerIpcJobCancelResponse {
        ok: response.ok,
        job_id: response.job_id,
        trace_id: response.trace_id.or(request.trace_id),
    })
}

#[tauri::command]
async fn crawler_job_status(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcJobStatusRequest,
) -> Result<CrawlerIpcJobStatusResponse, String> {
    state
        .lifecycle
        .ensure_running()
        .await
        .map_err(|error| error.to_string())?;
    let sidecar_request = CrawlerSidecarJobStatusRequest {
        trace_id: request.trace_id.clone(),
        job_id: request.job_id.clone(),
    };
    let response = state.gateway.job_status(sidecar_request).await?;
    Ok(CrawlerIpcJobStatusResponse {
        ok: response.ok,
        job_id: response.job_id,
        platform: response.platform,
        status: response.status,
        stop_reason: response.stop_reason,
        scanned_count: response.scanned_count,
        accepted_count: response.accepted_count,
        trace_id: response.trace_id.or(request.trace_id),
    })
}

#[tauri::command]
async fn crawler_job_logs(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcJobLogsRequest,
) -> Result<CrawlerIpcJobLogsResponse, String> {
    state
        .lifecycle
        .ensure_running()
        .await
        .map_err(|error| error.to_string())?;
    let sidecar_request = CrawlerSidecarJobLogsRequest {
        trace_id: request.trace_id.clone(),
        job_id: request.job_id.clone(),
    };
    let response = state.gateway.job_logs(sidecar_request).await?;
    Ok(CrawlerIpcJobLogsResponse {
        ok: response.ok,
        job_id: response.job_id,
        logs_json: response.logs_json,
        trace_id: response.trace_id.or(request.trace_id),
    })
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
        .invoke_handler(tauri::generate_handler![
            agent_ping,
            crawler_job_start,
            crawler_job_cancel,
            crawler_job_status,
            crawler_job_logs
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
