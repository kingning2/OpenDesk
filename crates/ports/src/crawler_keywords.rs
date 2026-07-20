//! Crawler keyword persistence port.

use crate::repository::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordImportResult {
    pub batch_id: String,
    pub inserted: i64,
    pub skipped_existing: i64,
    pub skipped_too_long: i64,
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordBatchSummary {
    pub batch_id: String,
    pub keyword_count: i64,
}

/// Keyword batches imported from CSV — Rust owns SQLite; Python never reads this.
pub trait CrawlerKeywordStore: Send + Sync {
    fn import_csv(
        &self,
        csv_content: &str,
        batch_id: Option<&str>,
    ) -> Result<KeywordImportResult, StoreError>;

    fn list_batches(&self) -> Result<Vec<KeywordBatchSummary>, StoreError>;

    fn enabled_keywords_for_batch(&self, batch_id: &str) -> Result<Vec<String>, StoreError>;
}
