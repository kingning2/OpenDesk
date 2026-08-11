use crate::contracts::ChannelCookie;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSidecarQrCheckResponse {
    pub ok: bool,
    pub status: String,
    pub session_id: Option<String>,
    pub cookies: Option<Vec<ChannelCookie>>,
    pub detail: Option<String>,
    pub trace_id: Option<String>,
}
