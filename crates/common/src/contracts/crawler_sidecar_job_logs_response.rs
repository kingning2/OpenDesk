use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerSidecarJobLogsResponse {
    pub ok: bool,
    pub job_id: String,
    pub logs_json: String,
    pub trace_id: Option<String>,
}
