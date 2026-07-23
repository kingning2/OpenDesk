use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcInboxUnmatchedListRequest {
    pub account_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
