use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSidecarLlmTestConnectionResponse {
    pub ok: bool,
    pub error_code: Option<String>,
    pub message: String,
    pub trace_id: Option<String>,
}
