use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcMessageMarkReadResponse {
    pub message_id: String,
    pub is_read: bool,
}
