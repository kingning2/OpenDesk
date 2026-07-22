use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcSendResponse {
    pub message_id: String,
    pub status: String,
}
