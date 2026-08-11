use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIpcQrCancelResponse {
    pub ok: bool,
    pub detail: Option<String>,
}
