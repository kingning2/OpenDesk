//! Crawler accepted-channel persistence port.
//!
//! 作者：coisini
//! 创建时间：2026-07-20

use crate::background_job::{EMAIL_STATUS_FOUND_API, EMAIL_STATUS_PENDING_ENRICH};
use crate::repository::StoreError;

/// One accepted channel row aligned with `crawler/channel_result` DTO.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelRecord {
    pub job_id: String,
    pub keyword: String,
    pub platform: String,
    pub channel_id: String,
    pub title: String,
    pub country: Option<String>,
    pub subscriber_count: Option<i64>,
    pub email: Option<String>,
    pub description: Option<String>,
    pub custom_url: Option<String>,
    pub email_status: String,
    pub enrich_attempts: i32,
    pub enrich_error: Option<String>,
    pub enriched_at: Option<String>,
}

/// Result of a Playwright email enrichment attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailEnrichResult {
    pub email: Option<String>,
    pub email_status: String,
    pub enrich_error: Option<String>,
}

impl ChannelRecord {
    /// Derive initial `email_status` from whether description already contained an email.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-20
    pub fn initial_email_status(email: &Option<String>) -> &'static str {
        match email.as_deref() {
            Some(value) if !value.trim().is_empty() => EMAIL_STATUS_FOUND_API,
            _ => EMAIL_STATUS_PENDING_ENRICH,
        }
    }
}

/// Accepted crawl results — Rust owns SQLite; survives job memory cleanup.
pub trait CrawlerChannelStore: Send + Sync {
    fn insert_accepted(&self, record: &ChannelRecord) -> Result<(), StoreError>;

    fn list_by_job(&self, job_id: &str) -> Result<Vec<ChannelRecord>, StoreError>;

    /// Mark a channel as actively being enriched by Worker.
    fn mark_enriching(&self, job_id: &str, channel_id: &str) -> Result<(), StoreError>;

    /// Persist enrichment outcome and bump attempt counter.
    fn apply_enrich_result(
        &self,
        job_id: &str,
        channel_id: &str,
        result: &EmailEnrichResult,
        enriched_at: &str,
    ) -> Result<(), StoreError>;
}
