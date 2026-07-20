//! Diesel-backed `crawler_channel` store.

use std::path::Path;

use diesel::prelude::*;
use ports::crawler_channels::{ChannelRecord, CrawlerChannelStore};
use ports::repository::StoreError;

use crate::crawler_db::schema::crawler_channel;
use crate::crawler_db::{CrawlerChannelRow, CrawlerDb, NewCrawlerChannel};

pub struct SqliteCrawlerChannelStore {
    db: CrawlerDb,
}

impl SqliteCrawlerChannelStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        Ok(Self {
            db: CrawlerDb::open(path)?,
        })
    }

    pub fn new(db: CrawlerDb) -> Self {
        Self { db }
    }
}

impl CrawlerChannelStore for SqliteCrawlerChannelStore {
    fn insert_accepted(&self, record: &ChannelRecord) -> Result<(), StoreError> {
        let new_row = NewCrawlerChannel {
            job_id: record.job_id.clone(),
            keyword: record.keyword.clone(),
            platform: record.platform.clone(),
            channel_id: record.channel_id.clone(),
            title: record.title.clone(),
            country: record.country.clone(),
            subscriber_count: record.subscriber_count,
            email: record.email.clone(),
            description: record.description.clone(),
            custom_url: record.custom_url.clone(),
        };

        self.db.with_conn(|conn| {
            let exists = crawler_channel::table
                .filter(crawler_channel::job_id.eq(&new_row.job_id))
                .filter(crawler_channel::channel_id.eq(&new_row.channel_id))
                .select(diesel::dsl::count_star())
                .first::<i64>(conn)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            if exists == 0 {
                diesel::insert_into(crawler_channel::table)
                    .values(&new_row)
                    .execute(conn)
                    .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            }
            Ok(())
        })
    }

    fn list_by_job(&self, job_id: &str) -> Result<Vec<ChannelRecord>, StoreError> {
        self.db.with_conn(|conn| {
            crawler_channel::table
                .filter(crawler_channel::job_id.eq(job_id))
                .select(CrawlerChannelRow::as_select())
                .order(crawler_channel::id.asc())
                .load::<CrawlerChannelRow>(conn)
                .map(|rows| rows.into_iter().map(ChannelRecord::from).collect())
                .map_err(|error| StoreError::Unavailable(error.to_string()))
        })
    }
}

impl From<CrawlerChannelRow> for ChannelRecord {
    fn from(row: CrawlerChannelRow) -> Self {
        Self {
            job_id: row.job_id,
            keyword: row.keyword,
            platform: row.platform,
            channel_id: row.channel_id,
            title: row.title,
            country: row.country,
            subscriber_count: row.subscriber_count,
            email: row.email,
            description: row.description,
            custom_url: row.custom_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn insert_and_list_roundtrip() {
        let dir = std::env::temp_dir().join(format!("crawler_ch_test_{}", Uuid::new_v4()));
        let path = dir.join("test.db");
        let store = SqliteCrawlerChannelStore::open(&path).expect("open");
        let record = ChannelRecord {
            job_id: "job-1".to_string(),
            keyword: "beauty".to_string(),
            platform: "youtube".to_string(),
            channel_id: "UC123".to_string(),
            title: "Test Channel".to_string(),
            country: Some("US".to_string()),
            subscriber_count: Some(1000),
            email: Some("a@b.com".to_string()),
            description: Some("contact a@b.com".to_string()),
            custom_url: Some("@test".to_string()),
        };
        store.insert_accepted(&record).expect("insert");
        let rows = store.list_by_job("job-1").expect("list");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].channel_id, "UC123");
        let _ = std::fs::remove_dir_all(dir);
    }
}
