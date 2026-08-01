use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerIpcKeywordsGenerateRequest {
    pub trace_id: Option<String>,
    pub directions: String,
    pub languages: String,
    pub count_per_language: i64,
    pub batch_id: Option<String>,
}
