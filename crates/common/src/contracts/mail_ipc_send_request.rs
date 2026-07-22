use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcSendRequest {
    pub customer_id: String,
    pub account_id: String,
    pub template_id: String,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
}
