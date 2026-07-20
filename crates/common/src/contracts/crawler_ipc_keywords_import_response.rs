use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerIpcKeywordsImportResponse {
    pub ok: bool,
    pub batch_id: String,
    pub inserted: i64,
    pub skipped_existing: i64,
    pub skipped_too_long: i64,
    pub total: i64,
    pub trace_id: Option<String>,
    pub message: Option<String>,
}
