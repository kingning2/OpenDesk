use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeIpcLlmSettingsSaveRequest {
    pub provider: String,
    pub base_url: Option<String>,
    pub model_id: String,
    pub api_key: String,
}
