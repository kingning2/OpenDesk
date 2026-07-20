use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerIpcKeywordsBatchesResponse {
    pub ok: bool,
    pub batches_json: String,
    pub trace_id: Option<String>,
}
