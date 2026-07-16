use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerEventJobFailed {
    pub event_id: String,
    pub occurred_at: String,
    pub job_id: String,
    pub platform: String,
    pub error_code: String,
    pub message: String,
}
