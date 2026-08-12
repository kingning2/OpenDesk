use crate::contracts::ChannelCookie;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIpcQrCheckResponse {
    pub ok: bool,
    pub status: String,
    pub session_id: Option<String>,
    pub cookies: Option<Vec<ChannelCookie>>,
    pub detail: Option<String>,
    pub qr_base64: Option<String>,
}
