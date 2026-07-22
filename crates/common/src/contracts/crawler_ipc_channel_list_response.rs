use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerIpcChannelListResponse {
    pub ok: bool,
    pub channels_json: String,
    pub total: i64,
    pub trace_id: Option<String>,
}
