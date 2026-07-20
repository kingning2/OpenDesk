//! Diesel-backed `crawler_keyword` store.

use std::path::Path;

use diesel::dsl::{count_star, max};
use diesel::prelude::*;
use ports::crawler_keywords::{CrawlerKeywordStore, KeywordBatchSummary, KeywordImportResult};
use ports::repository::StoreError;
use uuid::Uuid;

use crate::crawler_db::schema::crawler_keyword;
use crate::crawler_db::{CrawlerDb, NewCrawlerKeyword};

use super::csv_import::parse_csv;

pub struct SqliteCrawlerKeywordStore {
    db: CrawlerDb,
}

impl SqliteCrawlerKeywordStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        Ok(Self {
            db: CrawlerDb::open(path)?,
        })
    }

    pub fn new(db: CrawlerDb) -> Self {
        Self { db }
    }
}

impl CrawlerKeywordStore for SqliteCrawlerKeywordStore {
    fn import_csv(
        &self,
        csv_content: &str,
        batch_id: Option<&str>,
    ) -> Result<KeywordImportResult, StoreError> {
        let batch_id = batch_id
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let parsed = parse_csv(csv_content);
        let mut inserted = 0i64;
        let mut skipped_existing = 0i64;

        self.db.with_conn(|conn| {
            for row in parsed.rows {
                let enabled = i32::from(row.enabled);
                let already = crawler_keyword::table
                    .filter(crawler_keyword::batch_id.eq(&batch_id))
                    .filter(crawler_keyword::text.eq(&row.text))
                    .select(diesel::dsl::count_star())
                    .first::<i64>(conn)
                    .map_err(|error| StoreError::Unavailable(error.to_string()))?;
                if already > 0 {
                    skipped_existing += 1;
                    continue;
                }
                let new_row = NewCrawlerKeyword {
                    batch_id: batch_id.clone(),
                    text: row.text,
                    enabled,
                };
                diesel::insert_into(crawler_keyword::table)
                    .values(&new_row)
                    .execute(conn)
                    .map_err(|error| StoreError::Unavailable(error.to_string()))?;
                inserted += 1;
            }
            Ok(())
        })?;

        Ok(KeywordImportResult {
            batch_id,
            inserted,
            skipped_existing,
            skipped_too_long: parsed.skipped_too_long,
            total: parsed.total_unique,
        })
    }

    fn list_batches(&self) -> Result<Vec<KeywordBatchSummary>, StoreError> {
        self.db.with_conn(|conn| {
            crawler_keyword::table
                .group_by(crawler_keyword::batch_id)
                .select((crawler_keyword::batch_id, count_star()))
                .order(max(crawler_keyword::id).desc())
                .load::<(String, i64)>(conn)
                .map(|rows| {
                    rows.into_iter()
                        .map(|(batch_id, keyword_count)| KeywordBatchSummary {
                            batch_id,
                            keyword_count,
                        })
                        .collect()
                })
                .map_err(|error| StoreError::Unavailable(error.to_string()))
        })
    }

    fn enabled_keywords_for_batch(&self, batch_id: &str) -> Result<Vec<String>, StoreError> {
        self.db.with_conn(|conn| {
            crawler_keyword::table
                .filter(crawler_keyword::batch_id.eq(batch_id))
                .filter(crawler_keyword::enabled.eq(1))
                .select(crawler_keyword::text)
                .order(crawler_keyword::id.asc())
                .load::<String>(conn)
                .map_err(|error| StoreError::Unavailable(error.to_string()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn import_and_list_roundtrip() {
        let dir = std::env::temp_dir().join(format!("crawler_kw_test_{}", Uuid::new_v4()));
        let path = dir.join("test.db");
        let store = SqliteCrawlerKeywordStore::open(&path).expect("open");
        let result = store.import_csv("text\nalpha\nbeta", None).expect("import");
        assert_eq!(result.inserted, 2);
        let keywords = store
            .enabled_keywords_for_batch(&result.batch_id)
            .expect("list");
        assert_eq!(keywords, vec!["alpha".to_string(), "beta".to_string()]);
        let batches = store.list_batches().expect("batches");
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].keyword_count, 2);
        let _ = std::fs::remove_dir_all(dir);
    }
}
