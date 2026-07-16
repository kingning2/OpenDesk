use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerIpcJobStatusResponse {
    pub ok: bool,
    pub job_id: String,
    pub platform: String,
    pub status: String,
    pub stop_reason: Option<String>,
    pub scanned_count: Option<i64>,
    pub accepted_count: Option<i64>,
    pub trace_id: Option<String>,
}
