use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerIpcJobResultsResponse {
    pub ok: bool,
    pub job_id: String,
    pub results_json: String,
    pub trace_id: Option<String>,
}
