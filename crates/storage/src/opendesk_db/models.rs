//! Diesel row models for `opendesk.db` tables.

use diesel::prelude::*;

use super::schema::background_job;
use super::schema::customer;

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
