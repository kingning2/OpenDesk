use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcLinkInboundCustomerResponse {
    pub message_id: String,
}
