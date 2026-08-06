use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeIpcLlmSettingsGetResponse {
    pub provider: String,
    pub base_url: Option<String>,
    pub model_id: String,
    pub configured: bool,
    pub has_api_key: bool,
    pub tools_enabled: bool,
    pub memory_enabled: bool,
    pub knowledge_enabled: bool,
}
