use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcSendRequest {
    pub customer_id: Option<String>,
    pub to_address: String,
    pub account_id: String,
    pub template_id: Option<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub open_tracking_enabled: Option<bool>,
}
