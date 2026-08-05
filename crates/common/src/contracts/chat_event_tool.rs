use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatEventTool {
    pub event_id: String,
    pub occurred_at: String,
    pub session_id: String,
    pub message_id: String,
    pub seq: i64,
    pub name: String,
    pub arguments: String,
    pub ok: bool,
    pub result: Option<String>,
}
