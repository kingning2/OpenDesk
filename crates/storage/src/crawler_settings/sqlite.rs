//! Diesel-backed `crawler_setting` store.

use diesel::prelude::*;
use ports::crawler_settings::CrawlerSettingsStore;
use ports::repository::StoreError;

use crate::crawler_db::schema::crawler_setting;
use crate::crawler_db::CrawlerDb;

pub struct SqliteCrawlerSettingsStore {
    db: CrawlerDb,
}

impl SqliteCrawlerSettingsStore {
    pub fn new(db: CrawlerDb) -> Self {
        Self { db }
    }
}

impl CrawlerSettingsStore for SqliteCrawlerSettingsStore {
    fn get(&self, key: &str) -> Result<Option<String>, StoreError> {
        self.db.with_conn(|conn| {
            crawler_setting::table
                .filter(crawler_setting::key.eq(key))
                .select(crawler_setting::value)
                .first::<String>(conn)
                .optional()
                .map_err(|error| StoreError::Unavailable(error.to_string()))
        })
    }

    fn set(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.db.with_conn(|conn| {
            diesel::replace_into(crawler_setting::table)
                .values((
                    crawler_setting::key.eq(key),
                    crawler_setting::value.eq(value),
                ))
                .execute(conn)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ports::crawler_settings::YOUTUBE_API_KEY;
    use uuid::Uuid;

    #[test]
    fn set_and_get_roundtrip() {
        let dir = std::env::temp_dir().join(format!("crawler_settings_test_{}", Uuid::new_v4()));
        let path = dir.join("test.db");
        let db = CrawlerDb::open(&path).expect("open");
        let store = SqliteCrawlerSettingsStore::new(db);
        store.set(YOUTUBE_API_KEY, "test-key-123").expect("set");
        let value = store.get(YOUTUBE_API_KEY).expect("get");
        assert_eq!(value.as_deref(), Some("test-key-123"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
