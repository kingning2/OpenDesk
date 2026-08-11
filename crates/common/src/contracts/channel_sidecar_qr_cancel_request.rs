use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSidecarQrCancelRequest {
    pub session_id: String,
    pub trace_id: Option<String>,
}
