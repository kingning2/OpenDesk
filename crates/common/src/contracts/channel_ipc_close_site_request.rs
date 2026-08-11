use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIpcCloseSiteRequest {
    pub account_id: Option<String>,
}
