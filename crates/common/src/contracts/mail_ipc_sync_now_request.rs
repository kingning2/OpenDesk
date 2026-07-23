use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcSyncNowRequest {
    pub account_id: Option<String>,
}
