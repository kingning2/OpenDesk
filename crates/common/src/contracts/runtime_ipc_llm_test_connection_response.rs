use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeIpcLlmTestConnectionResponse {
    pub ok: bool,
    pub error_code: Option<String>,
    pub message: String,
}
