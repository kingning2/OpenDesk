use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcAccountListResponse {
    pub accounts_json: String,
    pub total: i64,
}
