use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerSidecarJobLogsRequest {
    pub trace_id: Option<String>,
    pub job_id: String,
}
