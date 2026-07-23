use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcLinkInboundCustomerRequest {
    pub message_id: String,
    pub customer_id: String,
}
