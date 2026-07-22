//! Mail persistence port for templates, accounts, and message history.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use crate::repository::StoreError;

/// Saved mail template record.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[derive(Debug, Clone)]
pub struct MailTemplateRecord {
    pub id: String,
    pub name: String,
    pub template_intent: String,
    pub subject_template: String,
    pub body_text_template: String,
    pub body_html_template: Option<String>,
    pub locale: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Saved mail account record.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[derive(Debug, Clone)]
pub struct MailAccountRecord {
    pub id: String,
    pub label: String,
    pub from_address: String,
    pub from_name: Option<String>,
    pub smtp_host: String,
    pub smtp_port: i64,
    pub use_tls: bool,
    pub username: String,
    pub password_ref: String,
    pub password_value: String,
    pub imap_host: Option<String>,
    pub imap_port: Option<i64>,
    pub imap_use_tls: Option<bool>,
    pub imap_sync_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Mutable mail account input from settings UI.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[derive(Debug, Clone)]
pub struct MailAccountWriteInput {
    pub id: Option<String>,
    pub label: String,
    pub from_address: String,
    pub from_name: Option<String>,
    pub smtp_host: String,
    pub smtp_port: i64,
    pub use_tls: bool,
    pub username: String,
    pub password: String,
    pub imap_host: Option<String>,
    pub imap_port: Option<i64>,
    pub imap_use_tls: Option<bool>,
    pub imap_sync_enabled: bool,
}

/// Persisted mail message record.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[derive(Debug, Clone)]
pub struct MailMessageRecord {
    pub id: String,
    pub customer_id: Option<String>,
    pub template_id: Option<String>,
    pub account_id: Option<String>,
    pub status: String,
    pub direction: String,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub error_message: Option<String>,
    pub sent_at: Option<String>,
    pub received_at: Option<String>,
    pub imap_uid: Option<i64>,
    pub imap_folder: Option<String>,
    pub rfc_message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Option<String>,
    pub is_favorite: bool,
    pub open_tracking_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Outbound message write input.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[derive(Debug, Clone)]
pub struct MailSendInput {
    pub customer_id: String,
    pub template_id: String,
    pub account_id: String,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub status: String,
    pub sent_at: Option<String>,
    pub rfc_message_id: Option<String>,
}

/// Manual inbound record write input.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[derive(Debug, Clone)]
pub struct MailInboundWriteInput {
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

/// Mail domain storage contract.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub trait MailStore: Send + Sync {
    fn list_templates(&self) -> Result<Vec<MailTemplateRecord>, StoreError>;

    fn get_template(&self, id: &str) -> Result<MailTemplateRecord, StoreError>;

    fn list_accounts(&self) -> Result<Vec<MailAccountRecord>, StoreError>;

    fn get_account(&self, id: &str) -> Result<MailAccountRecord, StoreError>;

    fn save_account(&self, input: MailAccountWriteInput) -> Result<MailAccountRecord, StoreError>;

    fn create_outbound_message(
        &self,
        input: MailSendInput,
    ) -> Result<MailMessageRecord, StoreError>;

    fn create_inbound_message(
        &self,
        input: MailInboundWriteInput,
    ) -> Result<MailMessageRecord, StoreError>;
}
