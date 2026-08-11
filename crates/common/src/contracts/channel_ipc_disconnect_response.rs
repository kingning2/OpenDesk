use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIpcDisconnectResponse {
    pub ok: bool,
}
