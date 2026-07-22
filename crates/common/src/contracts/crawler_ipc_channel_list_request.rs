use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerIpcChannelListRequest {
    pub trace_id: Option<String>,
    pub search: Option<String>,
    pub keyword: Option<String>,
    pub country: Option<String>,
    pub has_email: Option<bool>,
    pub email_status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
