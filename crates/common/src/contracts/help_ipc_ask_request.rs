use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpIpcAskRequest {
    pub message_id: Option<String>,
    pub text: String,
}
