use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmIpcChatResponse {
    pub reply: String,
    pub trace_id: Option<String>,
}
