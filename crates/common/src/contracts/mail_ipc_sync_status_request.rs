use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcSyncStatusRequest {
    pub account_id: Option<String>,
}
