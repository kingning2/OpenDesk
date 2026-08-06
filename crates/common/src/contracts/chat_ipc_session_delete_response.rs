use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIpcSessionDeleteResponse {
    pub ok: bool,
}
