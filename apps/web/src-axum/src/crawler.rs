//! 爬虫域 RPC helper：channel 列表 / 更新、job 启动 / 结果、settings。

use app_core::AppState;
use common::contracts::{
    CrawlerIpcChannelListRequest, CrawlerIpcChannelListResponse, CrawlerIpcChannelUpdateRequest,
    CrawlerIpcChannelUpdateResponse, CrawlerIpcJobResultsRequest, CrawlerIpcJobResultsResponse,
    CrawlerIpcJobStartRequest, CrawlerIpcJobStartResponse,
};
use common::i18n::Locale;
use ports::crawler_channels::{ChannelListQuery, CrawlerChannelStore};
use ports::crawler_settings::CrawlerSettingsStore;
use serde_json::{json, Value};

/// crawler settings 的 YouTube API key 常量（与桌面一致）。
const YOUTUBE_API_KEY: &str = "youtube_api_key";

/// channel 列表（分页 + 筛选），与桌面 `crawler_channel_list` 同构。
pub fn channel_list(
    store: &dyn CrawlerChannelStore,
    req: CrawlerIpcChannelListRequest,
) -> Result<CrawlerIpcChannelListResponse, String> {
    let query = ChannelListQuery {
        search: req.search,
        keyword: req.keyword,
        country: req.country,
        has_email: req.has_email,
        email_status: req.email_status,
        limit: req.limit.unwrap_or(50).clamp(1, 200),
        offset: req.offset.unwrap_or(0).max(0),
    };
    let result = store.list(query).map_err(|error| error.to_string())?;
    let payload: Vec<Value> = result
        .items
        .into_iter()
        .map(|row| {
            json!({
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
        trace_id: None,
    })
}

/// channel 更新验证邮箱。
pub fn channel_update(
    store: &dyn CrawlerChannelStore,
    req: CrawlerIpcChannelUpdateRequest,
) -> Result<CrawlerIpcChannelUpdateResponse, String> {
    let id = i32::try_from(req.id).map_err(|_| "invalid channel id".to_string())?;
    let row = store
        .update_verified_email(id, &req.verified_email)
        .map_err(|error| error.to_string())?;
    Ok(CrawlerIpcChannelUpdateResponse {
        ok: true,
        id: i64::from(row.id),
        verified_email: row.verified_email,
        email_status: Some(row.email_status),
        trace_id: None,
    })
}

/// job 启动：解析关键词后交给进程内 CrawlerService。
pub async fn job_start(app: &AppState, req: CrawlerIpcJobStartRequest) -> Result<Value, String> {
    let store = app.keywords_store.clone();
    let keywords_input = req.keywords.clone();
    let batch_id_input = req.batch_id.clone();
    let locale = Locale::parse(req.locale.as_deref());
    let keywords = tokio::task::spawn_blocking(move || {
        resolve_keywords(store.as_ref(), keywords_input, batch_id_input, locale)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;
    let keywords = keywords
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let response: CrawlerIpcJobStartResponse = app
        .crawler
        .start(req, keywords)
        .map_err(|error| error.to_string())?;
    Ok(json!(response))
}

/// 与桌面 `resolve_keywords` 等价的关键词解析。
fn resolve_keywords(
    store: &dyn ports::crawler_keywords::CrawlerKeywordStore,
    keywords: Option<String>,
    batch_id: Option<String>,
    locale: Locale,
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
        .ok_or_else(|| common::i18n::need_batch(locale))?;
    let list = store
        .enabled_keywords_for_batch(&batch)
        .map_err(|error| error.to_string())?;
    if list.is_empty() {
        return Err(common::i18n::empty_batch(locale, &batch));
    }
    Ok(list.join(","))
}

/// job 结果列表。
pub async fn job_results(
    app: &AppState,
    req: CrawlerIpcJobResultsRequest,
) -> Result<Value, String> {
    let store = app.channels_store.clone();
    let job_id = req.job_id.clone();
    let rows = tokio::task::spawn_blocking(move || {
        store
            .list_by_job(&job_id)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;
    let payload: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            json!({
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
    Ok(json!(CrawlerIpcJobResultsResponse {
        ok: true,
        job_id: req.job_id,
        results_json: serde_json::to_string(&payload).map_err(|error| error.to_string())?,
        trace_id: None,
    }))
}

/// 读取 YouTube API key（是否配置）。
pub fn youtube_api_key_get(store: &dyn CrawlerSettingsStore) -> Result<Value, String> {
    let configured = store
        .get(YOUTUBE_API_KEY)
        .map_err(|error| error.to_string())?
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    Ok(json!({ "ok": true, "configured": configured }))
}

/// 保存 YouTube API key。
pub fn youtube_api_key_set(store: &dyn CrawlerSettingsStore, req: Value) -> Result<Value, String> {
    let configured = req
        .get("configured")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let api_key = req
        .get("api_key")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if configured {
        if api_key.trim().is_empty() {
            return Err("API key is empty".to_string());
        }
        store
            .set(YOUTUBE_API_KEY, &api_key)
            .map_err(|error| error.to_string())?;
    } else {
        store
            .set(YOUTUBE_API_KEY, "")
            .map_err(|error| error.to_string())?;
    }
    Ok(json!({ "ok": true, "configured": configured }))
}
