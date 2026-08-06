use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIpcMessagesLoadResponse {
    pub messages_json: String,
}
