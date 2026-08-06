use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIpcSessionCreateResponse {
    pub session_json: String,
}
