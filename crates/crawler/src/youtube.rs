//! YouTube Data API 采集、筛选与频道持久化。

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use ports::background_job::{
    CrawlerEmailEnrichPayload, EMAIL_STATUS_PENDING_ENRICH, JOB_TYPE_CRAWLER_EMAIL_ENRICH,
};
use ports::crawler_channels::ChannelRecord;
use reqwest::blocking::Client;
use serde_json::Value;

use crate::job::JobHandle;
use crate::service::RunConfig;

const API_BASE: &str = "https://www.googleapis.com/youtube/v3";

#[derive(Debug)]
pub(super) enum CrawlError {
    QuotaExceeded,
    Cancelled,
    Message(String),
}

/// 采集单个关键词：搜索频道、按条件筛选并持久化结果。
#[allow(clippy::too_many_arguments)]
pub(super) fn crawl_keyword(
    client: &Client,
    handle: &Arc<JobHandle>,
    config: &RunConfig,
    keyword: &str,
    stop_flag: &Arc<AtomicBool>,
    stop_reason: &Arc<Mutex<Option<String>>>,
    scanned_total: &Arc<AtomicI64>,
    accepted_total: &Arc<AtomicI64>,
    search_pages: &Arc<AtomicI64>,
    channel_calls: &Arc<AtomicI64>,
    playlist_pages: &Arc<AtomicI64>,
) -> Result<(), CrawlError> {
    // 1. 初始化关键词进度，并在任何外部请求前响应取消。
    if stop_flag.load(Ordering::SeqCst) || handle.cancel_requested.load(Ordering::SeqCst) {
        return Err(CrawlError::Cancelled);
    }

    handle.push_log(
        &config.platform,
        "keyword_begin",
        format!("begin keyword={keyword}"),
        Some(keyword.to_string()),
        None,
    );

    let mut keyword_scanned = 0i64;
    let mut keyword_accepted = 0i64;
    let mut page_token: Option<String> = None;
    handle.set_progress(
        keyword,
        keyword_scanned,
        keyword_accepted,
        scanned_total.load(Ordering::SeqCst),
        accepted_total.load(Ordering::SeqCst),
        calculate_expected_quota(
            search_pages.load(Ordering::SeqCst),
            channel_calls.load(Ordering::SeqCst),
            playlist_pages.load(Ordering::SeqCst),
        ),
    );

    // 2. 逐页搜索频道，并批量补齐频道详情。
    loop {
        if stop_flag.load(Ordering::SeqCst) || handle.cancel_requested.load(Ordering::SeqCst) {
            return Err(CrawlError::Cancelled);
        }
        if reached_max_total(config, accepted_total.load(Ordering::SeqCst)) {
            set_stop_reason(stop_reason, "max_total_reached");
            stop_flag.store(true, Ordering::SeqCst);
            break;
        }

        sleep_rate(config.rate_limit_ms);
        search_pages.fetch_add(1, Ordering::SeqCst);
        let search_body = search_channels(client, config, keyword, page_token.as_deref())?;
        handle.push_log(
            &config.platform,
            "search_page",
            "search.list cost=100".to_string(),
            Some(keyword.to_string()),
            None,
        );

        let channel_ids = search_body
            .get("items")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        item.get("snippet")
                            .and_then(|value| value.get("channelId"))
                            .and_then(Value::as_str)
                            .or_else(|| {
                                item.get("id")
                                    .and_then(|value| value.get("channelId"))
                                    .and_then(Value::as_str)
                            })
                            .map(ToOwned::to_owned)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if channel_ids.is_empty() {
            page_token = next_page_token(&search_body);
            if page_token.is_none() {
                break;
            }
            continue;
        }

        sleep_rate(config.rate_limit_ms);
        channel_calls.fetch_add(1, Ordering::SeqCst);
        let channels_body = get_json(
            client,
            &config.api_key,
            "/channels",
            vec![
                ("part", "snippet,statistics,contentDetails".to_string()),
                ("id", channel_ids.join(",")),
            ],
        )?;
        let channel_items = channels_body
            .get("items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let batch_scanned = channel_items.len() as i64;
        keyword_scanned += batch_scanned;
        let global_scanned =
            scanned_total.fetch_add(batch_scanned, Ordering::SeqCst) + batch_scanned;
        handle.push_log(
            &config.platform,
            "channel_batch",
            format!("channels.list size={batch_scanned}"),
            Some(keyword.to_string()),
            None,
        );

        // 3. 过滤地区和年度活跃度，持久化符合条件的频道。
        for channel in channel_items {
            if stop_flag.load(Ordering::SeqCst) || handle.cancel_requested.load(Ordering::SeqCst) {
                return Err(CrawlError::Cancelled);
            }
            if reached_max_total(config, accepted_total.load(Ordering::SeqCst)) {
                set_stop_reason(stop_reason, "max_total_reached");
                stop_flag.store(true, Ordering::SeqCst);
                break;
            }

            let country = channel
                .get("snippet")
                .and_then(|value| value.get("country"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_uppercase();
            if !country.is_empty()
                && config
                    .exclude_countries
                    .iter()
                    .any(|value| value.eq_ignore_ascii_case(&country))
            {
                continue;
            }

            let uploads_id = channel
                .get("contentDetails")
                .and_then(|value| value.get("relatedPlaylists"))
                .and_then(|value| value.get("uploads"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let year_videos = if let Some(uploads_id) = uploads_id {
                sleep_rate(config.rate_limit_ms);
                let (count, used_pages) = count_year_videos(client, config, &uploads_id)?;
                playlist_pages.fetch_add(used_pages, Ordering::SeqCst);
                count
            } else {
                0
            };

            if year_videos < config.min_year_video_count {
                continue;
            }

            let record =
                youtube_channel_record(&channel, &config.job_id, keyword, &config.platform);
            if let Err(error) = config.channels.insert_accepted(&record) {
                handle.push_log(
                    &config.platform,
                    "filter",
                    format!("failed to persist channel: {error}"),
                    Some(keyword.to_string()),
                    None,
                );
            } else {
                handle.emit_channel_accepted(&record);
                maybe_enqueue_email_enrich(handle, config, &record);
            }

            keyword_accepted += 1;
            let global_accepted = accepted_total.fetch_add(1, Ordering::SeqCst) + 1;
            let quota_used = calculate_expected_quota(
                search_pages.load(Ordering::SeqCst),
                channel_calls.load(Ordering::SeqCst),
                playlist_pages.load(Ordering::SeqCst),
            );
            handle.set_progress(
                keyword,
                keyword_scanned,
                keyword_accepted,
                global_scanned,
                global_accepted,
                quota_used,
            );
        }

        // 4. 发布本页快照，并继续下一页或结束关键词。
        let quota_used = calculate_expected_quota(
            search_pages.load(Ordering::SeqCst),
            channel_calls.load(Ordering::SeqCst),
            playlist_pages.load(Ordering::SeqCst),
        );
        handle.set_progress(
            keyword,
            keyword_scanned,
            keyword_accepted,
            scanned_total.load(Ordering::SeqCst),
            accepted_total.load(Ordering::SeqCst),
            quota_used,
        );

        page_token = next_page_token(&search_body);
        if page_token.is_none() || stop_flag.load(Ordering::SeqCst) {
            break;
        }
    }

    handle.mark_keyword_done();
    handle.push_log(
        &config.platform,
        "keyword_done",
        format!(
            "keyword done accepted_total={}",
            accepted_total.load(Ordering::SeqCst)
        ),
        Some(keyword.to_string()),
        None,
    );
    Ok(())
}

fn search_channels(
    client: &Client,
    config: &RunConfig,
    keyword: &str,
    page_token: Option<&str>,
) -> Result<Value, CrawlError> {
    let mut query = vec![
        ("part", "snippet".to_string()),
        ("q", keyword.to_string()),
        ("type", "channel".to_string()),
        ("maxResults", "50".to_string()),
    ];
    if let Some(token) = page_token {
        query.push(("pageToken", token.to_string()));
    }
    get_json(client, &config.api_key, "/search", query)
}

fn count_year_videos(
    client: &Client,
    config: &RunConfig,
    uploads_playlist_id: &str,
) -> Result<(i64, i64), CrawlError> {
    let mut count = 0i64;
    let mut pages = 0i64;
    let mut page_token: Option<String> = None;

    loop {
        // 1. 按发布时间倒序读取上传列表。
        pages += 1;
        let mut query = vec![
            ("part", "contentDetails".to_string()),
            ("playlistId", uploads_playlist_id.to_string()),
            ("maxResults", "50".to_string()),
        ];
        if let Some(token) = &page_token {
            query.push(("pageToken", token.clone()));
        }
        let body = match get_json(client, &config.api_key, "/playlistItems", query) {
            Ok(value) => value,
            Err(CrawlError::Message(message)) if message.contains("playlistNotFound") => {
                return Ok((0, pages))
            }
            Err(error) => return Err(error),
        };

        // 2. 统计目标年份；遇到更早年份即可停止翻页。
        let items = body
            .get("items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for item in items {
            let Some(published) = item
                .get("contentDetails")
                .and_then(|value| value.get("videoPublishedAt"))
                .and_then(Value::as_str)
            else {
                continue;
            };
            let year = published
                .get(0..4)
                .and_then(|value| value.parse::<i32>().ok());
            match year {
                Some(value) if value < config.year => return Ok((count, pages)),
                Some(value) if value == config.year => count += 1,
                _ => {}
            }
        }

        page_token = next_page_token(&body);
        if page_token.is_none() {
            break;
        }
    }

    Ok((count, pages))
}

fn get_json(
    client: &Client,
    api_key: &str,
    path: &str,
    params: Vec<(&str, String)>,
) -> Result<Value, CrawlError> {
    let mut query: Vec<(String, String)> = params
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect();
    query.push(("key".to_string(), api_key.to_string()));

    let response = client
        .get(format!("{API_BASE}{path}"))
        .query(&query)
        .send()
        .map_err(|error| CrawlError::Message(format!("YouTube API network error: {error}")))?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| CrawlError::Message(format!("YouTube API read error: {error}")))?;

    if !status.is_success() {
        if is_quota_exceeded(&body) {
            return Err(CrawlError::QuotaExceeded);
        }
        return Err(CrawlError::Message(format!(
            "YouTube API HTTP {}: {}",
            status.as_u16(),
            truncate_body(&body)
        )));
    }

    serde_json::from_str(&body)
        .map_err(|error| CrawlError::Message(format!("invalid YouTube API JSON: {error}")))
}

pub(super) fn calculate_expected_quota(
    search_pages: i64,
    channel_calls: i64,
    playlist_item_pages: i64,
) -> i64 {
    (search_pages * 100) + channel_calls + playlist_item_pages
}

pub(super) fn reached_max_total(config: &RunConfig, accepted: i64) -> bool {
    config.max_total > 0 && accepted >= config.max_total
}

fn next_page_token(body: &Value) -> Option<String> {
    body.get("nextPageToken")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn is_quota_exceeded(body: &str) -> bool {
    let lowered = body.to_ascii_lowercase();
    lowered.contains("quotaexceeded") || lowered.contains("quota exceeded")
}

fn truncate_body(body: &str) -> String {
    body.chars().take(500).collect()
}

pub(super) fn set_stop_reason(target: &Arc<Mutex<Option<String>>>, reason: &str) {
    if let Ok(mut guard) = target.lock() {
        if guard.is_none() {
            *guard = Some(reason.to_string());
        }
    }
}

fn sleep_rate(rate_limit_ms: u64) {
    if rate_limit_ms > 0 {
        thread::sleep(Duration::from_millis(rate_limit_ms));
    }
}

fn youtube_channel_record(
    channel: &Value,
    job_id: &str,
    keyword: &str,
    platform: &str,
) -> ChannelRecord {
    let snippet = channel.get("snippet").and_then(Value::as_object);
    let statistics = channel.get("statistics").and_then(Value::as_object);
    let description = snippet
        .and_then(|value| value.get("description"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let subscriber_count = statistics
        .and_then(|value| value.get("subscriberCount"))
        .and_then(Value::as_str)
        .and_then(|value| value.parse::<i64>().ok());
    let email = extract_email(&description);
    let email_status = ChannelRecord::initial_email_status(&email).to_string();
    ChannelRecord {
        job_id: job_id.to_string(),
        keyword: keyword.to_string(),
        platform: platform.to_string(),
        channel_id: channel
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        title: snippet
            .and_then(|value| value.get("title"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        country: snippet
            .and_then(|value| value.get("country"))
            .and_then(Value::as_str)
            .map(str::to_string),
        subscriber_count,
        email,
        verified_email: None,
        description: Some(description),
        custom_url: snippet
            .and_then(|value| value.get("customUrl"))
            .and_then(Value::as_str)
            .map(str::to_string),
        email_status,
        enrich_attempts: 0,
        enrich_error: None,
        enriched_at: None,
    }
}

fn extract_email(description: &str) -> Option<String> {
    let normalized = description
        .replace("[at]", "@")
        .replace("(at)", "@")
        .replace("[dot]", ".")
        .replace("(dot)", ".");
    let mut token = String::new();
    for ch in normalized.chars() {
        if ch.is_whitespace() {
            if token.contains('@') && token.contains('.') && token.len() >= 5 {
                return Some(token);
            }
            token.clear();
        } else {
            token.push(ch);
        }
    }
    if token.contains('@') && token.contains('.') && token.len() >= 5 {
        Some(token)
    } else {
        None
    }
}

fn maybe_enqueue_email_enrich(handle: &JobHandle, config: &RunConfig, record: &ChannelRecord) {
    if record.email_status != EMAIL_STATUS_PENDING_ENRICH {
        return;
    }
    let Some(jobs_queue) = config.jobs_queue.as_ref() else {
        handle.push_log(
            &config.platform,
            "enrich_skip",
            format!(
                "email enrich queue unavailable; channel={} kept",
                record.channel_id
            ),
            Some(record.keyword.clone()),
            None,
        );
        return;
    };

    let payload = CrawlerEmailEnrichPayload {
        crawler_job_id: record.job_id.clone(),
        channel_id: record.channel_id.clone(),
        platform: record.platform.clone(),
        custom_url: record.custom_url.clone(),
        title: record.title.clone(),
        attempt: 1,
    };
    let payload_json = match serde_json::to_string(&payload) {
        Ok(value) => value,
        Err(error) => {
            handle.push_log(
                &config.platform,
                "enrich_skip",
                format!("enrich payload serialize failed: {error}"),
                Some(record.keyword.clone()),
                None,
            );
            return;
        }
    };

    match jobs_queue.enqueue(JOB_TYPE_CRAWLER_EMAIL_ENRICH, &payload_json) {
        Ok(job_id) => {
            handle.push_log(
                &config.platform,
                "enrich_enqueued",
                format!(
                    "email enrich queued background_job={job_id} channel={}",
                    record.channel_id
                ),
                Some(record.keyword.clone()),
                None,
            );
        }
        Err(error) => {
            // 邮箱补全是旁路能力，入队失败不能中断主采集流程。
            handle.push_log(
                &config.platform,
                "enrich_skip",
                format!(
                    "email enrich enqueue failed (non-blocking): {error}; channel={} kept",
                    record.channel_id
                ),
                Some(record.keyword.clone()),
                None,
            );
        }
    }
}
