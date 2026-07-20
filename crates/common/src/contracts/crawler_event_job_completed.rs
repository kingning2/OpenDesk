use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerEventJobCompleted {
    pub event_id: String,
    pub occurred_at: String,
    pub job_id: String,
    pub platform: String,
    pub stop_reason: String,
    pub scanned_count: i64,
    pub accepted_count: i64,
    pub quota_used: Option<i64>,
    pub duration_ms: Option<i64>,
}
