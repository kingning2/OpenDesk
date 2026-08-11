use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub id: String,
    pub conversation_id: String,
    pub direction: String,
    pub sender: String,
    pub content: String,
    pub created_at: String,
}
