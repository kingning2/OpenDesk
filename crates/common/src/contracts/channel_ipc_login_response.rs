use crate::contracts::ChannelCookie;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIpcLoginResponse {
    pub ok: bool,
    pub state: String,
    pub cookies: Option<Vec<ChannelCookie>>,
    pub detail: Option<String>,
}
