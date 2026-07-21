use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerIpcJobStartRequest {
    pub trace_id: Option<String>,
    pub platform: String,
    pub keywords: Option<String>,
    pub rate_limit_ms: Option<i64>,
    pub max_total: Option<i64>,
    pub year: Option<i64>,
    pub min_year_video_count: Option<i64>,
    pub exclude_countries: Option<String>,
    pub batch_id: Option<String>,
    pub api_key: String,
    pub locale: Option<String>,
}
