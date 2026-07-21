//! Background job queue port for Worker coordination.
//!
//! 作者：coisini
//! 创建时间：2026-07-20

use serde::{Deserialize, Serialize};

use crate::repository::StoreError;

/// Job type: Playwright/Chromium email enrichment for crawler channels.
pub const JOB_TYPE_CRAWLER_EMAIL_ENRICH: &str = "crawler_email_enrich";

pub const JOB_STATUS_QUEUED: &str = "queued";
pub const JOB_STATUS_RUNNING: &str = "running";
pub const JOB_STATUS_COMPLETED: &str = "completed";
pub const JOB_STATUS_FAILED: &str = "failed";
pub const JOB_STATUS_CANCELLED: &str = "cancelled";

/// Email enrichment lifecycle for crawler channels.
pub const EMAIL_STATUS_FOUND_API: &str = "found_api";
pub const EMAIL_STATUS_PENDING_ENRICH: &str = "pending_enrich";
pub const EMAIL_STATUS_ENRICHING: &str = "enriching";
pub const EMAIL_STATUS_FOUND_PLAYWRIGHT: &str = "found_playwright";
pub const EMAIL_STATUS_NOT_FOUND: &str = "not_found";
pub const EMAIL_STATUS_ENRICH_FAILED: &str = "enrich_failed";

/// One `background_job` row exposed to Worker handlers.
#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundJobRecord {
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

/// Payload for `crawler_email_enrich` jobs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrawlerEmailEnrichPayload {
    pub crawler_job_id: String,
    pub channel_id: String,
    pub platform: String,
    pub custom_url: Option<String>,
    pub title: String,
    pub attempt: i32,
}

/// Queue operations for `opendesk.db.background_job`.
pub trait BackgroundJobStore: Send + Sync {
    /// Insert a queued job and return its id.
    fn enqueue(&self, job_type: &str, payload_json: &str) -> Result<String, StoreError>;

    /// Atomically claim the oldest queued job, optionally filtered by type.
    fn claim_next(&self, job_type: Option<&str>)
        -> Result<Option<BackgroundJobRecord>, StoreError>;

    /// Mark a job completed.
    fn mark_completed(&self, job_id: &str) -> Result<(), StoreError>;

    /// Mark a job failed with an error message.
    fn mark_failed(&self, job_id: &str, error_message: &str) -> Result<(), StoreError>;
}
