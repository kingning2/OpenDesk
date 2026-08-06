use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIpcMessagesLoadRequest {
    pub session_id: String,
}
