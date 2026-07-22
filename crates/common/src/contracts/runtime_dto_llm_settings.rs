use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDtoLlmSettings {
    pub provider: String,
    pub base_url: Option<String>,
    pub model_id: String,
    pub configured: bool,
    pub has_api_key: bool,
}
