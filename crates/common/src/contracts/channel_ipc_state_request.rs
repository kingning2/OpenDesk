use crate::contracts::{ChannelAccount, ChannelSettings};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIpcStateRequest {
    pub accounts: Vec<ChannelAccount>,
    pub settings: ChannelSettings,
}
