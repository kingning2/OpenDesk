use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerEventJobProgress {
    pub event_id: String,
    pub occurred_at: String,
    pub job_id: String,
    pub platform: String,
    pub status: Option<String>,
    pub message: Option<String>,
    pub stop_reason: Option<String>,
    pub current_keyword: Option<String>,
    pub scanned_count: i64,
    pub accepted_count: i64,
    pub quota_used: Option<i64>,
    pub search_pages: Option<i64>,
    pub keyword_scanned: Option<i64>,
    pub keyword_accepted: Option<i64>,
    pub keywords_total: Option<i64>,
    pub keywords_done: Option<i64>,
    pub keyword_stats_json: Option<String>,
    pub error_message: Option<String>,
}
