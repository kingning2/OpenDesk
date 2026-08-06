use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpIpcAskResponse {
    pub ok: bool,
    pub message_id: String,
    pub error_message: Option<String>,
}
