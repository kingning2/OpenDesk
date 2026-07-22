use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcTemplateListResponse {
    pub templates_json: String,
    pub total: i64,
}
