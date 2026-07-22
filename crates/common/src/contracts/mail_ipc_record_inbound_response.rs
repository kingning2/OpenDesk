use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcRecordInboundResponse {
    pub message_id: String,
}
