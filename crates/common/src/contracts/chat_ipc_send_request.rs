use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIpcSendRequest {
    pub trace_id: Option<String>,
    pub message_id: Option<String>,
    pub session_id: String,
    pub messages_json: Option<String>,
    pub text: String,
}
