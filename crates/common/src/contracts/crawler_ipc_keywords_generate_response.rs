use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerIpcKeywordsGenerateResponse {
    pub ok: bool,
    pub batch_id: String,
    pub inserted: i64,
    pub requested: i64,
    pub keywords_json: String,
    pub trace_id: Option<String>,
    pub message: Option<String>,
}
