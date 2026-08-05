use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIpcSendResponse {
    pub ok: bool,
    pub session_id: String,
    pub message_id: String,
    pub error_message: Option<String>,
}
