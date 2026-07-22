//! 爬虫频道列表 Tauri IPC command（持久化 `crawler_channel` 分页查询）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-22

use common::contracts::{
    CrawlerIpcChannelListRequest, CrawlerIpcChannelListResponse, CrawlerIpcChannelUpdateRequest,
    CrawlerIpcChannelUpdateResponse,
};
use ports::crawler_channels::{ChannelListQuery, CrawlerChannelStore};

use crate::state::AppState;

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 200;

/// List persisted crawler channels with optional filters and pagination.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
///
/// # 参数
/// - `state` — 应用共享状态
/// - `request` — 查询条件（search / keyword / country / has_email / email_status / limit / offset）
///
/// # 返回值
/// `channels_json` 数组字符串与 `total` 总数。
#[tauri::command]
pub async fn crawler_channel_list(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcChannelListRequest,
) -> Result<CrawlerIpcChannelListResponse, String> {
    let store = state.channels_store.clone();
    let limit = request.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = request.offset.unwrap_or(0).max(0);
    let trace_id = request.trace_id.clone();
    let query = ChannelListQuery {
        search: request.search,
        keyword: request.keyword,
        country: request.country,
        has_email: request.has_email,
        email_status: request.email_status,
        limit,
        offset,
    };
    let result = tauri::async_runtime::spawn_blocking(move || store.list(query))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    let payload: Vec<serde_json::Value> = result
        .items
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.id,
                "job_id": row.job_id,
                "keyword": row.keyword,
                "platform": row.platform,
                "channel_id": row.channel_id,
                "title": row.title,
                "country": row.country,
                "subscriber_count": row.subscriber_count,
                "email": row.email,
                "verified_email": row.verified_email,
                "description": row.description,
                "custom_url": row.custom_url,
                "email_status": row.email_status,
                "enrich_attempts": row.enrich_attempts,
                "enrich_error": row.enrich_error,
                "enriched_at": row.enriched_at,
            })
        })
        .collect();
    Ok(CrawlerIpcChannelListResponse {
        ok: true,
        channels_json: serde_json::to_string(&payload).map_err(|error| error.to_string())?,
        total: result.total,
        trace_id,
    })
}

/// Update human-verified email for one persisted crawler channel row.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[tauri::command]
pub async fn crawler_channel_update(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcChannelUpdateRequest,
) -> Result<CrawlerIpcChannelUpdateResponse, String> {
    let store = state.channels_store.clone();
    let id = i32::try_from(request.id).map_err(|_| "invalid channel id".to_string())?;
    let verified_email = request.verified_email.clone();
    let trace_id = request.trace_id.clone();
    let row = tauri::async_runtime::spawn_blocking(move || {
        CrawlerChannelStore::update_verified_email(store.as_ref(), id, &verified_email)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;
    Ok(CrawlerIpcChannelUpdateResponse {
        ok: true,
        id: i64::from(row.id),
        verified_email: row.verified_email,
        email_status: Some(row.email_status),
        trace_id,
    })
}
