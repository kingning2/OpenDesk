use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerSidecarJobCancelRequest {
    pub trace_id: Option<String>,
    pub job_id: String,
}
