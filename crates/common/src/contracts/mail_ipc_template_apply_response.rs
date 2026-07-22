use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcTemplateApplyResponse {
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
}
