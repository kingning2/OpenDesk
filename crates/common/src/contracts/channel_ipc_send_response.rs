use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIpcSendResponse {
    pub ok: bool,
    pub message_id: String,
    pub detail: Option<String>,
}
