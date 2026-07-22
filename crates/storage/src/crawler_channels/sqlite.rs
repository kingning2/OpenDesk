//! Diesel-backed `crawler_channel` store.

use std::path::Path;

use diesel::prelude::*;
use ports::background_job::{EMAIL_STATUS_ENRICHING, EMAIL_STATUS_VERIFIED_MANUAL};
use ports::crawler_channels::{
    ChannelListQuery, ChannelListRecord, ChannelListResult, ChannelRecord, CrawlerChannelStore,
    EmailEnrichResult,
};
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
            email_status: record.email_status.clone(),
            enrich_attempts: record.enrich_attempts,
            enrich_error: record.enrich_error.clone(),
            enriched_at: record.enriched_at.clone(),
            verified_email: record.verified_email.clone(),
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

    fn list(&self, query: ChannelListQuery) -> Result<ChannelListResult, StoreError> {
        self.db.with_conn(|conn| {
            let search = query
                .search
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| format!("%{value}%"));
            let keyword = query
                .keyword
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            let country = query
                .country
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            let email_status = query
                .email_status
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);

            let build_query = || {
                let mut list_query = crawler_channel::table.into_boxed();
                if let Some(pattern) = search.as_deref() {
                    list_query = list_query.filter(
                        crawler_channel::title
                            .like(pattern)
                            .or(crawler_channel::email.like(pattern))
                            .or(crawler_channel::keyword.like(pattern))
                            .or(crawler_channel::channel_id.like(pattern)),
                    );
                }
                if let Some(value) = keyword.as_deref() {
                    list_query = list_query.filter(crawler_channel::keyword.eq(value));
                }
                if let Some(value) = country.as_deref() {
                    list_query = list_query.filter(crawler_channel::country.eq(value));
                }
                if let Some(true) = query.has_email {
                    list_query = list_query.filter(
                        crawler_channel::email
                            .is_not_null()
                            .and(crawler_channel::email.ne("")),
                    );
                }
                if let Some(false) = query.has_email {
                    list_query = list_query.filter(
                        crawler_channel::email
                            .is_null()
                            .or(crawler_channel::email.eq("")),
                    );
                }
                if let Some(value) = email_status.as_deref() {
                    list_query = list_query.filter(crawler_channel::email_status.eq(value));
                }
                list_query
            };

            let total = build_query()
                .count()
                .get_result::<i64>(conn)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;

            let rows = build_query()
                .order(crawler_channel::id.desc())
                .limit(query.limit)
                .offset(query.offset)
                .select(CrawlerChannelRow::as_select())
                .load::<CrawlerChannelRow>(conn)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;

            Ok(ChannelListResult {
                items: rows.into_iter().map(ChannelListRecord::from).collect(),
                total,
            })
        })
    }

    fn mark_enriching(&self, job_id: &str, channel_id: &str) -> Result<(), StoreError> {
        self.db.with_conn(|conn| {
            diesel::update(
                crawler_channel::table
                    .filter(crawler_channel::job_id.eq(job_id))
                    .filter(crawler_channel::channel_id.eq(channel_id)),
            )
            .set(crawler_channel::email_status.eq(EMAIL_STATUS_ENRICHING))
            .execute(conn)
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(())
        })
    }

    fn apply_enrich_result(
        &self,
        job_id: &str,
        channel_id: &str,
        result: &EmailEnrichResult,
        enriched_at: &str,
    ) -> Result<(), StoreError> {
        self.db.with_conn(|conn| {
            diesel::update(
                crawler_channel::table
                    .filter(crawler_channel::job_id.eq(job_id))
                    .filter(crawler_channel::channel_id.eq(channel_id)),
            )
            .set((
                crawler_channel::email.eq(result.email.clone()),
                crawler_channel::email_status.eq(result.email_status.clone()),
                crawler_channel::enrich_error.eq(result.enrich_error.clone()),
                crawler_channel::enriched_at.eq(Some(enriched_at.to_string())),
                crawler_channel::enrich_attempts.eq(crawler_channel::enrich_attempts + 1),
            ))
            .execute(conn)
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(())
        })
    }

    fn update_verified_email(
        &self,
        id: i32,
        verified_email: &str,
    ) -> Result<ChannelListRecord, StoreError> {
        let normalized = verified_email.trim();
        let stored_email = if normalized.is_empty() {
            None
        } else {
            Some(normalized.to_string())
        };
        let next_status = stored_email
            .as_ref()
            .map(|_| EMAIL_STATUS_VERIFIED_MANUAL.to_string());

        self.db.with_conn(|conn| {
            let updated = if let Some(status) = next_status {
                diesel::update(crawler_channel::table.filter(crawler_channel::id.eq(id)))
                    .set((
                        crawler_channel::verified_email.eq(stored_email.clone()),
                        crawler_channel::email_status.eq(status),
                    ))
                    .execute(conn)
            } else {
                diesel::update(crawler_channel::table.filter(crawler_channel::id.eq(id)))
                    .set(crawler_channel::verified_email.eq::<Option<String>>(None))
                    .execute(conn)
            }
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            if updated == 0 {
                return Err(StoreError::NotFound);
            }
            crawler_channel::table
                .filter(crawler_channel::id.eq(id))
                .select(CrawlerChannelRow::as_select())
                .first::<CrawlerChannelRow>(conn)
                .map(ChannelListRecord::from)
                .map_err(|error| StoreError::Unavailable(error.to_string()))
        })
    }
}

impl From<CrawlerChannelRow> for ChannelListRecord {
    fn from(row: CrawlerChannelRow) -> Self {
        Self {
            id: row.id,
            job_id: row.job_id,
            keyword: row.keyword,
            platform: row.platform,
            channel_id: row.channel_id,
            title: row.title,
            country: row.country,
            subscriber_count: row.subscriber_count,
            email: row.email,
            verified_email: row.verified_email,
            description: row.description,
            custom_url: row.custom_url,
            email_status: row.email_status,
            enrich_attempts: row.enrich_attempts,
            enrich_error: row.enrich_error,
            enriched_at: row.enriched_at,
        }
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
            verified_email: row.verified_email,
            description: row.description,
            custom_url: row.custom_url,
            email_status: row.email_status,
            enrich_attempts: row.enrich_attempts,
            enrich_error: row.enrich_error,
            enriched_at: row.enriched_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ports::background_job::{EMAIL_STATUS_FOUND_PLAYWRIGHT, EMAIL_STATUS_PENDING_ENRICH};
    use uuid::Uuid;

    fn sample_record() -> ChannelRecord {
        ChannelRecord {
            job_id: "job-1".to_string(),
            keyword: "beauty".to_string(),
            platform: "youtube".to_string(),
            channel_id: "UC123".to_string(),
            title: "Test Channel".to_string(),
            country: Some("US".to_string()),
            subscriber_count: Some(1000),
            email: Some("a@b.com".to_string()),
            verified_email: None,
            description: Some("contact a@b.com".to_string()),
            custom_url: Some("@test".to_string()),
            email_status: ports::background_job::EMAIL_STATUS_FOUND_API.to_string(),
            enrich_attempts: 0,
            enrich_error: None,
            enriched_at: None,
        }
    }

    #[test]
    fn insert_and_list_roundtrip() {
        let dir = std::env::temp_dir().join(format!("crawler_ch_test_{}", Uuid::new_v4()));
        let path = dir.join("test.db");
        let store = SqliteCrawlerChannelStore::open(&path).expect("open");
        store.insert_accepted(&sample_record()).expect("insert");
        let rows = store.list_by_job("job-1").expect("list");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].channel_id, "UC123");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn apply_enrich_result_updates_email_status() {
        let dir = std::env::temp_dir().join(format!("crawler_ch_test_{}", Uuid::new_v4()));
        let path = dir.join("test.db");
        let store = SqliteCrawlerChannelStore::open(&path).expect("open");
        let mut record = sample_record();
        record.email = None;
        record.email_status = EMAIL_STATUS_PENDING_ENRICH.to_string();
        store.insert_accepted(&record).expect("insert");
        store
            .mark_enriching("job-1", "UC123")
            .expect("mark enriching");
        store
            .apply_enrich_result(
                "job-1",
                "UC123",
                &EmailEnrichResult {
                    email: Some("found@example.com".to_string()),
                    email_status: EMAIL_STATUS_FOUND_PLAYWRIGHT.to_string(),
                    enrich_error: None,
                },
                "12345",
            )
            .expect("apply");
        let rows = store.list_by_job("job-1").expect("list");
        assert_eq!(rows[0].email.as_deref(), Some("found@example.com"));
        assert_eq!(rows[0].email_status, EMAIL_STATUS_FOUND_PLAYWRIGHT);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn list_supports_search_and_has_email_filter() {
        let dir = std::env::temp_dir().join(format!("crawler_ch_test_{}", Uuid::new_v4()));
        let path = dir.join("test.db");
        let store = SqliteCrawlerChannelStore::open(&path).expect("open");
        store.insert_accepted(&sample_record()).expect("insert");
        let mut without_email = sample_record();
        without_email.channel_id = "UC456".to_string();
        without_email.title = "Other Channel".to_string();
        without_email.email = None;
        without_email.email_status = EMAIL_STATUS_PENDING_ENRICH.to_string();
        store.insert_accepted(&without_email).expect("insert");

        let all = store
            .list(ChannelListQuery {
                search: None,
                keyword: None,
                country: None,
                has_email: None,
                email_status: None,
                limit: 50,
                offset: 0,
            })
            .expect("list all");
        assert_eq!(all.total, 2);

        let with_email = store
            .list(ChannelListQuery {
                search: Some("Test".to_string()),
                keyword: None,
                country: None,
                has_email: Some(true),
                email_status: None,
                limit: 50,
                offset: 0,
            })
            .expect("list with email");
        assert_eq!(with_email.total, 1);
        assert_eq!(with_email.items[0].channel_id, "UC123");

        let _ = std::fs::remove_dir_all(dir);
    }
}
