use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailDtoImapSyncState {
    pub account_id: String,
    pub folder: String,
    pub uidvalidity: Option<i64>,
    pub last_uid: i64,
    pub last_sync_at: Option<String>,
    pub last_error: Option<String>,
    pub full_synced: bool,
    pub is_syncing: Option<bool>,
}
