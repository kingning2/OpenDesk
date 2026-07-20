use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerSidecarJobStartResponse {
    pub ok: bool,
    pub job_id: String,
    pub trace_id: Option<String>,
}
