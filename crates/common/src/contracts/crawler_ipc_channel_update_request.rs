use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerIpcChannelUpdateRequest {
    pub trace_id: Option<String>,
    pub id: i64,
    pub verified_email: String,
}
