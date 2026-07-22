use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcTemplateApplyRequest {
    pub customer_id: String,
    pub template_id: String,
    pub account_id: Option<String>,
}
