use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerEventJobLog {
    pub event_id: String,
    pub occurred_at: String,
    pub job_id: String,
    pub platform: String,
    pub seq: i64,
    pub phase: String,
    pub level: String,
    pub message: String,
    pub keyword: Option<String>,
    pub detail: Option<String>,
}
