use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIpcSessionListResponse {
    pub sessions_json: String,
}
