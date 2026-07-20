//! Crawler settings persistence port (API keys, etc.).

use crate::repository::StoreError;

pub const YOUTUBE_API_KEY: &str = "youtube_api_key";

/// Key-value settings owned by Rust SQLite (`crawler_setting` table).
pub trait CrawlerSettingsStore: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, StoreError>;

    fn set(&self, key: &str, value: &str) -> Result<(), StoreError>;
}
