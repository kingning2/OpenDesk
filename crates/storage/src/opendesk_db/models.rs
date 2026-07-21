//! Diesel row models for `opendesk.db` tables.

use diesel::prelude::*;

use super::schema::background_job;

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
