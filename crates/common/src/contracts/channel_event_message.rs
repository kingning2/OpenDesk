use crate::contracts::ChannelMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelEventMessage {
    pub account_id: String,
    pub message: ChannelMessage,
    pub suggestion: Option<String>,
}
