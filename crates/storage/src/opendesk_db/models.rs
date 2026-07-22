//! Diesel row models for `opendesk.db` tables.

use diesel::prelude::*;

use super::schema::background_job;
use super::schema::customer;
use super::schema::mail_account;
use super::schema::mail_message;
use super::schema::mail_template;
use super::schema::script_snippet;

/// Insertable row for `background_job`.
#[derive(Debug, Insertable)]
#[diesel(table_name = background_job)]
pub struct NewBackgroundJob {
    pub id: String,
    pub job_type: String,
    pub payload_json: String,
    pub status: String,
    pub progress: f32,
    pub error_message: Option<String>,
    pub worker_pid: Option<i32>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Queryable `background_job` row.
#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = background_job)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct BackgroundJobRow {
    pub id: String,
    pub job_type: String,
    pub payload_json: String,
    pub status: String,
    pub progress: f32,
    pub error_message: Option<String>,
    pub worker_pid: Option<i32>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Insertable row for `customer`.
#[derive(Debug, Insertable)]
#[diesel(table_name = customer)]
pub struct NewCustomerRow {
    pub id: String,
    pub display_name: Option<String>,
    pub email: String,
    pub whatsapp_phone: Option<String>,
    pub source_channel: String,
    pub source_meta: Option<String>,
    pub lifecycle_status: String,
    pub outreach_stage: String,
    pub quoted_price: Option<f32>,
    pub quoted_currency: Option<String>,
    pub quoted_at: Option<String>,
    pub pricing_tier: Option<String>,
    pub cooperation_status: String,
    pub package_name: Option<String>,
    pub monthly_fee: Option<f32>,
    pub contract_start: Option<String>,
    pub contract_end: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Queryable `customer` row.
#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = customer)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CustomerRow {
    pub id: String,
    pub display_name: Option<String>,
    pub email: String,
    pub whatsapp_phone: Option<String>,
    pub source_channel: String,
    pub source_meta: Option<String>,
    pub lifecycle_status: String,
    pub outreach_stage: String,
    pub quoted_price: Option<f32>,
    pub quoted_currency: Option<String>,
    pub quoted_at: Option<String>,
    pub pricing_tier: Option<String>,
    pub cooperation_status: String,
    pub package_name: Option<String>,
    pub monthly_fee: Option<f32>,
    pub contract_start: Option<String>,
    pub contract_end: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Insertable row for `mail_template`.
#[derive(Debug, Insertable)]
#[diesel(table_name = mail_template)]
pub struct NewMailTemplateRow {
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

/// Queryable `mail_template` row.
#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = mail_template)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MailTemplateRow {
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

/// Insertable row for `mail_account`.
#[derive(Debug, Insertable)]
#[diesel(table_name = mail_account)]
pub struct NewMailAccountRow {
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

/// Queryable `mail_account` row.
#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = mail_account)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MailAccountRow {
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

/// Insertable row for `mail_message`.
#[derive(Debug, Insertable)]
#[diesel(table_name = mail_message)]
pub struct NewMailMessageRow {
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
    pub references_header: Option<String>,
    pub is_favorite: bool,
    pub open_tracking_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Insertable row for `script_snippet`.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[derive(Debug, Insertable)]
#[diesel(table_name = script_snippet)]
pub struct NewScriptSnippetRow {
    pub id: String,
    pub source_id: String,
    pub title: String,
    pub stage: Option<String>,
    pub trigger_text: Option<String>,
    pub description: Option<String>,
    pub from_stage: Option<String>,
    pub to_stage: Option<String>,
    pub tags_json: String,
    pub body_text: String,
    pub category_l1: Option<String>,
    pub category_l2: Option<String>,
    pub needs_boss_input: bool,
    pub boss_input_hint: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Queryable `script_snippet` row.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = script_snippet)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ScriptSnippetRow {
    pub id: String,
    pub source_id: String,
    pub title: String,
    pub stage: Option<String>,
    pub trigger_text: Option<String>,
    pub description: Option<String>,
    pub from_stage: Option<String>,
    pub to_stage: Option<String>,
    pub tags_json: String,
    pub body_text: String,
    pub category_l1: Option<String>,
    pub category_l2: Option<String>,
    pub needs_boss_input: bool,
    pub boss_input_hint: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Queryable `mail_message` row.
#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = mail_message)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MailMessageRow {
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
    pub references_header: Option<String>,
    pub is_favorite: bool,
    pub open_tracking_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
