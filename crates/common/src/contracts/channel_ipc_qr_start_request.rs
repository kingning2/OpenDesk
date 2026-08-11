use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIpcQrStartRequest {
    pub account_id: String,
}
