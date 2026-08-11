use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIpcConnectResponse {
    pub ok: bool,
    pub state: String,
    pub detail: Option<String>,
}
