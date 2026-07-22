use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcMessageListResponse {
    pub messages_json: String,
    pub total: i64,
}
