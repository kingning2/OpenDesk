use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIpcSendRequest {
    pub conversation_id: String,
    pub content: String,
}
