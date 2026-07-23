use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcSyncStatusResponse {
    pub items_json: String,
    pub total: i64,
}
