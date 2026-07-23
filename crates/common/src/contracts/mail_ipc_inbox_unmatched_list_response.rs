use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcInboxUnmatchedListResponse {
    pub messages_json: String,
    pub total: i64,
}
