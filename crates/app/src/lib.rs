//! Tauri shell: agent / license IPC plus in-process Rust crawler commands.

mod logging;
mod state;

use adapter::agent_sidecar::RuntimeAgentSidecar;
use agent::app::ping::PingAgent;
use common::contracts::{
    AgentIpcPingRequest, AgentIpcPingResponse, CrawlerIpcJobCancelRequest,
    CrawlerIpcJobCancelResponse, CrawlerIpcJobLogsRequest, CrawlerIpcJobLogsResponse,
    CrawlerIpcJobResultsRequest, CrawlerIpcJobResultsResponse, CrawlerIpcJobStartRequest,
    CrawlerIpcJobStartResponse, CrawlerIpcJobStatusRequest, CrawlerIpcJobStatusResponse,
    CrawlerIpcKeywordsBatchesResponse, CrawlerIpcKeywordsImportRequest,
    CrawlerIpcKeywordsImportResponse,
};
use common::license::{LicenseActivateRequest, LicenseStatus};
use crawler::CrawlerService;
use kernel::event::{EventBus, InMemoryEventBus};
use logging::init_tracing;
use ports::crawler_channels::CrawlerChannelStore;
use ports::crawler_keywords::CrawlerKeywordStore;
use ports::crawler_settings::{CrawlerSettingsStore, YOUTUBE_API_KEY};
use runtime::sidecar::lifecycle::{SidecarConfig, SidecarLifecycle};
use state::{build_license_gate, AppState};
use std::path::PathBuf;
use std::sync::Arc;
use storage::crawler_channels::SqliteCrawlerChannelStore;
use storage::crawler_db::CrawlerDb;
use storage::crawler_keywords::SqliteCrawlerKeywordStore;
use storage::crawler_settings::SqliteCrawlerSettingsStore;
use storage::opendesk_db::OpendeskDb;
use tauri::Manager;

fn crawler_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path.push("crawler.db");
    path
}

fn opendesk_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path.push("opendesk.db");
    path
}

/// Resolve comma-separated keywords from the request or SQLite keyword batch.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-20
///
/// # 参数
///
/// * `store` - 关键词存储
/// * `keywords` - 直接传入的关键词串
/// * `batch_id` - 批次 ID
/// * `locale` - UI 语言（错误文案由后端翻译）
fn resolve_keywords(
    store: &dyn CrawlerKeywordStore,
    keywords: Option<String>,
    batch_id: Option<String>,
    locale: crawler::Locale,
) -> Result<String, String> {
    if let Some(text) = keywords.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }) {
        return Ok(text);
    }
    let batch = batch_id
        .and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .ok_or_else(|| crawler::need_batch(locale))?;
    let list = store
        .enabled_keywords_for_batch(&batch)
        .map_err(|error| error.to_string())?;
    if list.is_empty() {
        return Err(crawler::empty_batch(locale, &batch));
    }
    Ok(list.join(","))
}

/// Agent ping IPC；有锁构建下会先执行授权硬检查。
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
#[tauri::command]
async fn license_status(state: tauri::State<'_, AppState>) -> Result<LicenseStatus, String> {
    state
        .license
        .status()
        .await
        .map_err(|error| error.to_string())
}

/// 读取本机设备码 IPC。
#[tauri::command]
async fn license_machine_code(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state
        .license
        .machine_code()
        .await
        .map_err(|error| error.to_string())
}

/// 激活授权 IPC。
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

/// Start a crawl job in the Rust `CrawlerService` (no Python sidecar).
#[tauri::command]
async fn crawler_job_start(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcJobStartRequest,
) -> Result<CrawlerIpcJobStartResponse, String> {
    let store = state.keywords_store.clone();
    let keywords_input = request.keywords.clone();
    let batch_id_input = request.batch_id.clone();
    let locale = crawler::Locale::parse(request.locale.as_deref());
    let keywords = tauri::async_runtime::spawn_blocking(move || {
        resolve_keywords(store.as_ref(), keywords_input, batch_id_input, locale)
    })
    .await
    .map_err(|error| error.to_string())??;
    let keywords = keywords
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    state.crawler.start(request, keywords)
}

#[tauri::command]
async fn crawler_keywords_import(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcKeywordsImportRequest,
) -> Result<CrawlerIpcKeywordsImportResponse, String> {
    let store = state.keywords_store.clone();
    let csv_content = request.csv_content.clone();
    let batch_id = request.batch_id.clone();
    let trace_id = request.trace_id.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        store
            .import_csv(&csv_content, batch_id.as_deref())
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())??;
    Ok(CrawlerIpcKeywordsImportResponse {
        ok: true,
        batch_id: result.batch_id,
        inserted: result.inserted,
        skipped_existing: result.skipped_existing,
        skipped_too_long: result.skipped_too_long,
        total: result.total,
        trace_id,
        message: None,
    })
}

#[tauri::command]
async fn crawler_keywords_batches(
    state: tauri::State<'_, AppState>,
) -> Result<CrawlerIpcKeywordsBatchesResponse, String> {
    let store = state.keywords_store.clone();
    let batches = tauri::async_runtime::spawn_blocking(move || {
        store.list_batches().map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())??;
    let payload: Vec<serde_json::Value> = batches
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "batch_id": item.batch_id,
                "keyword_count": item.keyword_count,
            })
        })
        .collect();
    Ok(CrawlerIpcKeywordsBatchesResponse {
        ok: true,
        batches_json: serde_json::to_string(&payload).map_err(|error| error.to_string())?,
        trace_id: None,
    })
}

#[tauri::command]
async fn crawler_job_cancel(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcJobCancelRequest,
) -> Result<CrawlerIpcJobCancelResponse, String> {
    state.crawler.cancel(request)
}

#[tauri::command]
async fn crawler_job_status(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcJobStatusRequest,
) -> Result<CrawlerIpcJobStatusResponse, String> {
    state.crawler.status(request)
}

#[tauri::command]
async fn crawler_job_logs(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcJobLogsRequest,
) -> Result<CrawlerIpcJobLogsResponse, String> {
    state.crawler.logs(request)
}

/// List accepted channels for a job from SQLite (`crawler_channel`).
#[tauri::command]
async fn crawler_job_results(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcJobResultsRequest,
) -> Result<CrawlerIpcJobResultsResponse, String> {
    let store = state.channels_store.clone();
    let job_id = request.job_id.clone();
    let trace_id = request.trace_id.clone();
    let rows = tauri::async_runtime::spawn_blocking(move || {
        store
            .list_by_job(&job_id)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())??;
    let payload: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "keyword": row.keyword,
                "platform": row.platform,
                "channel_id": row.channel_id,
                "title": row.title,
                "country": row.country,
                "subscriber_count": row.subscriber_count,
                "email": row.email,
                "description": row.description,
                "custom_url": row.custom_url,
                "email_status": row.email_status,
                "enrich_attempts": row.enrich_attempts,
                "enrich_error": row.enrich_error,
                "enriched_at": row.enriched_at,
            })
        })
        .collect();
    Ok(CrawlerIpcJobResultsResponse {
        ok: true,
        job_id: request.job_id,
        results_json: serde_json::to_string(&payload).map_err(|error| error.to_string())?,
        trace_id,
    })
}

#[derive(serde::Serialize)]
struct CrawlerYoutubeApiKeyResponse {
    api_key: String,
}

#[derive(serde::Deserialize)]
struct CrawlerYoutubeApiKeySetRequest {
    api_key: String,
}

/// Read persisted YouTube Data API key from SQLite.
#[tauri::command]
async fn crawler_youtube_api_key_get(
    state: tauri::State<'_, AppState>,
) -> Result<CrawlerYoutubeApiKeyResponse, String> {
    let store = state.settings_store.clone();
    let api_key = tauri::async_runtime::spawn_blocking(move || {
        store
            .get(YOUTUBE_API_KEY)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())??
    .unwrap_or_default();
    Ok(CrawlerYoutubeApiKeyResponse { api_key })
}

/// Persist YouTube Data API key to SQLite.
#[tauri::command]
async fn crawler_youtube_api_key_set(
    state: tauri::State<'_, AppState>,
    request: CrawlerYoutubeApiKeySetRequest,
) -> Result<CrawlerYoutubeApiKeyResponse, String> {
    let store = state.settings_store.clone();
    let api_key = request.api_key.trim().to_string();
    let value = api_key.clone();
    tauri::async_runtime::spawn_blocking(move || {
        store
            .set(YOUTUBE_API_KEY, &value)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())??;
    Ok(CrawlerYoutubeApiKeyResponse { api_key })
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
    let db_path = crawler_db_path();
    let _opendesk_db = OpendeskDb::open(opendesk_db_path()).expect("open opendesk database");
    let crawler_db = CrawlerDb::open(&db_path).expect("open crawler database");
    let channels_store = Arc::new(SqliteCrawlerChannelStore::new(crawler_db.clone()))
        as Arc<dyn CrawlerChannelStore>;
    let settings_store = Arc::new(SqliteCrawlerSettingsStore::new(crawler_db.clone()))
        as Arc<dyn CrawlerSettingsStore>;
    let crawler = Arc::new(CrawlerService::new(channels_store.clone()));
    let keywords_store =
        Arc::new(SqliteCrawlerKeywordStore::new(crawler_db)) as Arc<dyn CrawlerKeywordStore>;
    let app_state = AppState {
        lifecycle: lifecycle.clone(),
        gateway,
        license,
        crawler,
        keywords_store,
        channels_store,
        settings_store,
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
            license_activate,
            crawler_job_start,
            crawler_job_cancel,
            crawler_job_status,
            crawler_job_logs,
            crawler_job_results,
            crawler_keywords_import,
            crawler_keywords_batches,
            crawler_youtube_api_key_get,
            crawler_youtube_api_key_set
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
