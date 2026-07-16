use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerSidecarJobStatusResponse {
    pub ok: bool,
    pub job_id: String,
    pub platform: String,
    pub status: String,
    pub stop_reason: Option<String>,
    pub message: Option<String>,
    pub current_keyword: Option<String>,
    pub scanned_count: Option<i64>,
    pub accepted_count: Option<i64>,
    pub keyword_scanned: Option<i64>,
    pub keyword_accepted: Option<i64>,
    pub quota_used: Option<i64>,
    pub keyword_stats_json: Option<String>,
    pub error_message: Option<String>,
    pub trace_id: Option<String>,
}
