use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailDtoMailMessage {
    pub id: String,
    pub customer_id: Option<String>,
    pub template_id: Option<String>,
    pub account_id: Option<String>,
    pub status: String,
    pub direction: String,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub to_address: Option<String>,
    pub from_address: Option<String>,
    pub error_message: Option<String>,
    pub sent_at: Option<String>,
    pub received_at: Option<String>,
    pub imap_uid: Option<i64>,
    pub imap_folder: Option<String>,
    pub rfc_message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Option<String>,
    pub is_favorite: Option<bool>,
    pub open_tracking_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
