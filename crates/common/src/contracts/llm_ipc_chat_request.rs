use crate::contracts::{LlmMessage, LlmProvider};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmIpcChatRequest {
    pub messages: Vec<LlmMessage>,
    pub provider: LlmProvider,
    pub trace_id: Option<String>,
}
