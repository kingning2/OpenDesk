use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIpcOpenSiteRequest {
    pub account_id: String,
}
