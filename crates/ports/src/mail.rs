//! Mail persistence port for templates, accounts, and message history.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use crate::repository::StoreError;

/// OS keyring service name shared with other OpenDesk secrets.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub const MAIL_KEYRING_SERVICE: &str = "OpenDesk";

/// Build keyring user id for one mail account password.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
///
/// # 参数
///
/// * `account_id` - Mail account id
///
/// # 返回值
///
/// Keyring account name (`mail_account/{id}`).
pub fn mail_keyring_user(account_id: &str) -> String {
    format!("mail_account/{account_id}")
}

/// Build DB `password_ref` pointing at OS keyring.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
///
/// # 参数
///
/// * `account_id` - Mail account id
///
/// # 返回值
///
/// Opaque password reference string without the secret itself.
pub fn mail_password_ref(account_id: &str) -> String {
    format!(
        "keyring:{MAIL_KEYRING_SERVICE}/{}",
        mail_keyring_user(account_id)
    )
}

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
    pub to_address: Option<String>,
    pub from_address: Option<String>,
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
    pub customer_id: Option<String>,
    pub to_address: String,
    pub template_id: Option<String>,
    pub template_name: String,
    pub account_id: String,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub sent_at: Option<String>,
    pub rfc_message_id: Option<String>,
    pub from_address: String,
    pub open_tracking_id: Option<String>,
}

/// Custom template create/update input.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[derive(Debug, Clone)]
pub struct MailTemplateWriteInput {
    pub id: Option<String>,
    pub name: String,
    pub template_intent: String,
    pub subject_template: String,
    pub body_text_template: String,
    pub body_html_template: Option<String>,
    pub locale: Option<String>,
    pub is_active: bool,
    pub sort_order: i64,
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

/// Local mailbox list filter.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[derive(Debug, Clone)]
pub struct MailMessageListFilter {
    pub direction: String,
    pub account_id: Option<String>,
    pub customer_id: Option<String>,
    pub query: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

/// Mail domain storage contract.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub trait MailStore: Send + Sync {
    fn list_templates(&self) -> Result<Vec<MailTemplateRecord>, StoreError>;

    fn get_template(&self, id: &str) -> Result<MailTemplateRecord, StoreError>;

    /// Create or update a custom (non-system) template.
    fn save_template(
        &self,
        input: MailTemplateWriteInput,
    ) -> Result<MailTemplateRecord, StoreError>;

    fn list_accounts(&self) -> Result<Vec<MailAccountRecord>, StoreError>;

    fn get_account(&self, id: &str) -> Result<MailAccountRecord, StoreError>;

    fn save_account(&self, input: MailAccountWriteInput) -> Result<MailAccountRecord, StoreError>;

    /// Resolve plaintext SMTP password from keyring (or legacy inline column).
    ///
    /// # 注意事项
    ///
    /// Caller must not log the returned secret.
    fn resolve_account_password(&self, account_id: &str) -> Result<String, StoreError>;

    fn create_outbound_message(
        &self,
        input: MailSendInput,
    ) -> Result<MailMessageRecord, StoreError>;

    fn create_inbound_message(
        &self,
        input: MailInboundWriteInput,
    ) -> Result<MailMessageRecord, StoreError>;

    /// List local messages for inbox/sent workbench.
    fn list_messages(
        &self,
        filter: MailMessageListFilter,
    ) -> Result<(Vec<MailMessageRecord>, i64), StoreError>;
}
