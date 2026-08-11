use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSidecarQrCancelResponse {
    pub ok: bool,
    pub detail: Option<String>,
    pub trace_id: Option<String>,
}
