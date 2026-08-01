use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcMessageMarkReadRequest {
    pub message_id: String,
}
