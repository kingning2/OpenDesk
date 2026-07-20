//! Crawler accepted-channel persistence port.

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
}

/// Accepted crawl results — Rust owns SQLite; survives job memory cleanup.
pub trait CrawlerChannelStore: Send + Sync {
    fn insert_accepted(&self, record: &ChannelRecord) -> Result<(), StoreError>;

    fn list_by_job(&self, job_id: &str) -> Result<Vec<ChannelRecord>, StoreError>;
}
