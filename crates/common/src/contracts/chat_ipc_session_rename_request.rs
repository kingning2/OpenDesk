use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIpcSessionRenameRequest {
    pub id: String,
    pub title: String,
}
