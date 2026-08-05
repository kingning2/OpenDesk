use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatEventToken {
    pub event_id: String,
    pub occurred_at: String,
    pub session_id: String,
    pub message_id: String,
    pub seq: i64,
    pub delta: String,
    pub reasoning: Option<String>,
    pub done: Option<bool>,
    pub error_message: Option<String>,
}
