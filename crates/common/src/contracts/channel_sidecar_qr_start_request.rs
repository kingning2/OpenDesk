use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSidecarQrStartRequest {
    pub account_id: String,
    pub trace_id: Option<String>,
}
