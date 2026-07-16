use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerSidecarJobCancelResponse {
    pub ok: bool,
    pub job_id: String,
    pub trace_id: Option<String>,
}
