//! 爬虫关键词批次 Tauri commands（import / list batches）。
//!
//! 作者：coisini
//! 创建时间：2026-07-21

use common::contracts::{
    CrawlerIpcKeywordsBatchesResponse, CrawlerIpcKeywordsGenerateRequest,
    CrawlerIpcKeywordsGenerateResponse, CrawlerIpcKeywordsImportRequest,
    CrawlerIpcKeywordsImportResponse,
};
use ports::crawler_keywords::CrawlerKeywordStore;

use crate::app::commands::llm::stored_llm_client;
use crate::app::state::AppState;

/// Resolve comma-separated keywords from the request or SQLite keyword batch.
///
/// 作者：coisini
/// 创建时间：2026-07-20
///
/// # 参数
/// - `store` — 关键词存储
/// - `keywords` — 直接传入的关键词串
/// - `batch_id` — 批次 ID
/// - `locale` — UI 语言（错误文案由后端翻译）
///
/// # 返回值
/// 逗号分隔关键词；批次为空或不存在时返回本地化错误。
pub(crate) fn resolve_keywords(
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

/// Import keywords from CSV text into Rust SQLite.
///
/// 作者：coisini
/// 创建时间：2026-07-20
///
/// # 参数
/// - `state` — 应用共享状态
/// - `request` — CSV 导入请求
///
/// # 返回值
/// 导入统计（inserted / skipped）。
#[tauri::command]
pub async fn crawler_keywords_import(
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

/// List keyword import batches.
///
/// 作者：coisini
/// 创建时间：2026-07-20
///
/// # 参数
/// - `state` — 应用共享状态
///
/// # 返回值
/// `batches_json` 批次列表。
#[tauri::command]
pub async fn crawler_keywords_batches(
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

/// 使用 Rust LLM 生成关键词并写入新批次。
#[tauri::command]
pub async fn crawler_keywords_generate(
    state: tauri::State<'_, AppState>,
    request: CrawlerIpcKeywordsGenerateRequest,
) -> Result<CrawlerIpcKeywordsGenerateResponse, String> {
    // 1. 规范化输入并执行 crawler 域内的关键词生成用例。
    let directions = split_csv(&request.directions);
    let languages = split_csv(&request.languages);
    let count_per_language = usize::try_from(request.count_per_language.max(1))
        .unwrap_or(usize::MAX)
        .min(200);
    let client = stored_llm_client(state.inner()).await?;
    let (keywords, requested) = crawler::GenerateCrawlerKeywords::execute(
        &client,
        &directions,
        &languages,
        count_per_language,
    )
    .await?;

    // 2. 同步生成结果到现有关键词批次存储。
    let keywords_json = serde_json::to_string(&keywords).map_err(|error| error.to_string())?;
    let csv_content = keywords.join("\n");
    let store = state.keywords_store.clone();
    let batch_id = request.batch_id.clone();
    let imported = tauri::async_runtime::spawn_blocking(move || {
        store
            .import_csv(&csv_content, batch_id.as_deref())
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())??;

    // 3. 保持现有 IPC 的批次统计与 JSON 字符串语义。
    Ok(CrawlerIpcKeywordsGenerateResponse {
        ok: true,
        batch_id: imported.batch_id,
        inserted: imported.inserted,
        requested: i64::try_from(requested).unwrap_or(i64::MAX),
        keywords_json,
        trace_id: request.trace_id,
        message: Some("ok".to_string()),
    })
}

fn split_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}
