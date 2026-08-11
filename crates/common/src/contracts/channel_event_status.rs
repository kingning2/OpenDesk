use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelEventStatus {
    pub account_id: String,
    pub state: String,
    pub detail: Option<String>,
}
