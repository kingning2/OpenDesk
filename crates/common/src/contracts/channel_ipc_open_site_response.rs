use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIpcOpenSiteResponse {
    pub ok: bool,
    pub detail: Option<String>,
}
