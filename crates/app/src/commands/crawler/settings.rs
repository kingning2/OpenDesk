//! 爬虫设置 Tauri commands（YouTube API key 读写）。
//!
//! 作者：coisini
//! 创建时间：2026-07-21

use ports::crawler_settings::YOUTUBE_API_KEY;

use crate::state::AppState;

/// YouTube API key 读取响应。
///
/// 作者：coisini
/// 创建时间：2026-07-20
#[derive(serde::Serialize)]
pub struct CrawlerYoutubeApiKeyResponse {
    /// 已持久化的 API key；未设置时为空串。
    pub api_key: String,
}

/// YouTube API key 写入请求。
///
/// 作者：coisini
/// 创建时间：2026-07-20
#[derive(serde::Deserialize)]
pub struct CrawlerYoutubeApiKeySetRequest {
    /// 待写入的 API key（会 trim）。
    pub api_key: String,
}

/// Read persisted YouTube Data API key from SQLite.
///
/// 作者：coisini
/// 创建时间：2026-07-20
///
/// # 参数
/// - `state` — 应用共享状态
///
/// # 返回值
/// 当前 API key；未配置时 `api_key` 为空。
#[tauri::command]
pub async fn crawler_youtube_api_key_get(
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
///
/// 作者：coisini
/// 创建时间：2026-07-20
///
/// # 参数
/// - `state` — 应用共享状态
/// - `request` — 含新 API key
///
/// # 返回值
/// 写入后的 API key 回显。
#[tauri::command]
pub async fn crawler_youtube_api_key_set(
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
