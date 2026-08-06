use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDtoMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub thinking: Option<String>,
    pub tools_json: Option<String>,
    pub seq: i64,
    pub created_at: i64,
}
