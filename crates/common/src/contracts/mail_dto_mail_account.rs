use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailDtoMailAccount {
    pub id: String,
    pub label: String,
    pub from_address: String,
    pub from_name: Option<String>,
    pub smtp_host: String,
    pub smtp_port: i64,
    pub use_tls: bool,
    pub username: String,
    pub password_ref: String,
    pub imap_host: Option<String>,
    pub imap_port: Option<i64>,
    pub imap_use_tls: Option<bool>,
    pub imap_sync_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}
