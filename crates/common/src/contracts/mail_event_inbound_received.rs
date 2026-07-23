use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailEventInboundReceived {
    pub event_id: String,
    pub occurred_at: String,
    pub message_id: String,
    pub account_id: String,
    pub customer_id: Option<String>,
    pub direction: String,
    pub subject: Option<String>,
    pub from_address: Option<String>,
}
