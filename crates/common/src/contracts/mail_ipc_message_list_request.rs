use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcMessageListRequest {
    pub direction: String,
    pub account_id: Option<String>,
    pub customer_id: Option<String>,
    pub query: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
