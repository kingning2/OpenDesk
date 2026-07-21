//! 爬虫 job 生命周期 Tauri commands（start / cancel / status / logs / results）。
//!
//! 作者：coisini
//! 创建时间：2026-07-21

use common::contracts::{
    CrawlerIpcJobCancelRequest, CrawlerIpcJobCancelResponse, CrawlerIpcJobLogsRequest,
    CrawlerIpcJobLogsResponse, CrawlerIpcJobResultsRequest, CrawlerIpcJobResultsResponse,
    CrawlerIpcJobStartRequest, CrawlerIpcJobStartResponse, CrawlerIpcJobStatusRequest,
    CrawlerIpcJobStatusResponse,
};

use super::keywords::resolve_keywords;
use crate::state::AppState;

/// Start a crawl job in the Rust `CrawlerService` (no Python sidecar).
///
/// 作者：coisini
/// 创建时间：2026-07-16
///
/// # 参数
/// - `state` — 应用共享状态
/// - `request` — 启动请求（platform / batch / api_key 等）
///
/// # 返回值
/// 含 `job_id` 的启动响应；关键词解析失败时返回错误。
#[tauri::command]
pub async fn crawler_job_start(
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

/// Cancel a running crawl job.
///
/// 作者：coisini
/// 创建时间：2026-07-16
///
/// # 参数
/// - `state` — 应用共享状态
/// - `request` — 取消请求（含 `job_id`）
///
/// # 返回值
/// 取消确认响应。
#[tauri::command]
pub async fn crawler_job_cancel(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcJobCancelRequest,
) -> Result<CrawlerIpcJobCancelResponse, String> {
    state.crawler.cancel(request)
}

/// Query crawl job status (debug / fallback; UI prefers Event push).
///
/// 作者：coisini
/// 创建时间：2026-07-16
///
/// # 参数
/// - `state` — 应用共享状态
/// - `request` — 状态查询请求
///
/// # 返回值
/// 当前 job 状态快照。
#[tauri::command]
pub async fn crawler_job_status(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcJobStatusRequest,
) -> Result<CrawlerIpcJobStatusResponse, String> {
    state.crawler.status(request)
}

/// Fetch crawl process logs (debug / fallback; UI prefers Event push).
///
/// 作者：coisini
/// 创建时间：2026-07-16
///
/// # 参数
/// - `state` — 应用共享状态
/// - `request` — 日志查询请求
///
/// # 返回值
/// `logs_json` 数组字符串。
#[tauri::command]
pub async fn crawler_job_logs(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcJobLogsRequest,
) -> Result<CrawlerIpcJobLogsResponse, String> {
    state.crawler.logs(request)
}

/// List accepted channels for a job from SQLite (`crawler_channel`).
///
/// 作者：coisini
/// 创建时间：2026-07-20
///
/// # 参数
/// - `state` — 应用共享状态
/// - `request` — 结果查询请求（含 `job_id`）
///
/// # 返回值
/// `results_json` 频道数组字符串。
#[tauri::command]
pub async fn crawler_job_results(
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
