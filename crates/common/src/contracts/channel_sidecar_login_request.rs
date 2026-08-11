use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSidecarLoginRequest {
    pub account_id: String,
    pub credential: String,
    pub trace_id: Option<String>,
}
