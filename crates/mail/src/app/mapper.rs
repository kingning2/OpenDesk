//! Mappers between mail store records and IPC JSON payloads.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use ports::mail::{MailAccountRecord, MailMessageRecord, MailTemplateRecord};
use serde_json::json;

/// Serialize mail templates to JSON for IPC responses.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
///
/// # 参数
///
/// * `records` - Template records loaded from the store
///
/// # 返回值
///
/// JSON string containing template DTO objects.
pub fn templates_to_json(records: &[MailTemplateRecord]) -> Result<String, String> {
    serde_json::to_string(
        &records
            .iter()
            .map(|record| {
                json!({
                    "id": record.id,
                    "name": record.name,
                    "template_intent": record.template_intent,
                    "subject_template": record.subject_template,
                    "body_text_template": record.body_text_template,
                    "body_html_template": record.body_html_template,
                    "locale": record.locale,
                    "is_system": record.is_system,
                    "is_active": record.is_active,
                    "sort_order": record.sort_order,
                    "created_at": record.created_at,
                    "updated_at": record.updated_at,
                })
            })
            .collect::<Vec<_>>(),
    )
    .map_err(|error| error.to_string())
}

/// Serialize mail accounts to JSON for IPC responses.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
///
/// # 参数
///
/// * `records` - Account records loaded from the store
///
/// # 返回值
///
/// JSON string containing account DTO objects without plaintext password.
pub fn accounts_to_json(records: &[MailAccountRecord]) -> Result<String, String> {
    serde_json::to_string(
        &records
            .iter()
            .map(|record| {
                json!({
                    "id": record.id,
                    "label": record.label,
                    "from_address": record.from_address,
                    "from_name": record.from_name,
                    "smtp_host": record.smtp_host,
                    "smtp_port": record.smtp_port,
                    "use_tls": record.use_tls,
                    "username": record.username,
                    "password_ref": record.password_ref,
                    "imap_host": record.imap_host,
                    "imap_port": record.imap_port,
                    "imap_use_tls": record.imap_use_tls,
                    "imap_sync_enabled": record.imap_sync_enabled,
                    "created_at": record.created_at,
                    "updated_at": record.updated_at,
                })
            })
            .collect::<Vec<_>>(),
    )
    .map_err(|error| error.to_string())
}

/// Serialize mail messages to JSON for IPC responses.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
///
/// # 参数
///
/// * `records` - Message records loaded from the store
///
/// # 返回值
///
/// JSON string containing mail_message DTO objects.
pub fn messages_to_json(records: &[MailMessageRecord]) -> Result<String, String> {
    serde_json::to_string(
        &records
            .iter()
            .map(|record| {
                json!({
                    "id": record.id,
                    "customer_id": record.customer_id,
                    "template_id": record.template_id,
                    "account_id": record.account_id,
                    "status": record.status,
                    "direction": record.direction,
                    "subject": record.subject,
                    "body_text": record.body_text,
                    "body_html": record.body_html,
                    "to_address": record.to_address,
                    "from_address": record.from_address,
                    "error_message": record.error_message,
                    "sent_at": record.sent_at,
                    "received_at": record.received_at,
                    "imap_uid": record.imap_uid,
                    "imap_folder": record.imap_folder,
                    "rfc_message_id": record.rfc_message_id,
                    "in_reply_to": record.in_reply_to,
                    "references": record.references,
                    "is_favorite": record.is_favorite,
                    "open_tracking_id": record.open_tracking_id,
                    "created_at": record.created_at,
                    "updated_at": record.updated_at,
                })
            })
            .collect::<Vec<_>>(),
    )
    .map_err(|error| error.to_string())
}
