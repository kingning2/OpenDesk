use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcRecordInboundRequest {
    pub customer_id: String,
    pub from_address: String,
    pub from_name: Option<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub received_at: String,
    pub rfc_message_id: Option<String>,
    pub in_reply_to: Option<String>,
}
