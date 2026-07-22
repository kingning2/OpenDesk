use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerIpcChannelUpdateResponse {
    pub ok: bool,
    pub id: i64,
    pub verified_email: Option<String>,
    pub email_status: Option<String>,
    pub trace_id: Option<String>,
}
