use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAccount {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub api_key: String,
    pub default_model: Option<String>,
}
