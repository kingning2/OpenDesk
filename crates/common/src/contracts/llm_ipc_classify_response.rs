use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmIpcClassifyResponse {
    pub intent: String,
    pub trace_id: Option<String>,
}
