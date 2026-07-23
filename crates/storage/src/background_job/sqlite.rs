//! Diesel-backed `background_job` store.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use diesel::prelude::*;
use ports::background_job::{BackgroundJobRecord, BackgroundJobStore, JOB_STATUS_QUEUED};
use ports::repository::StoreError;
use uuid::Uuid;

use crate::opendesk_db::schema::background_job;
use crate::opendesk_db::{BackgroundJobRow, NewBackgroundJob, OpendeskDb};

/// SQLite implementation of [`BackgroundJobStore`].
pub struct SqliteBackgroundJobStore {
    db: OpendeskDb,
}

impl SqliteBackgroundJobStore {
    /// Open `opendesk.db` and return a job store handle.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-20
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        Ok(Self {
            db: OpendeskDb::open(path)?,
        })
    }

    pub fn new(db: OpendeskDb) -> Self {
        Self { db }
    }
}

impl BackgroundJobStore for SqliteBackgroundJobStore {
    fn enqueue(&self, job_type: &str, payload_json: &str) -> Result<String, StoreError> {
        let id = Uuid::new_v4().to_string();
        let row = NewBackgroundJob {
            id: id.clone(),
            job_type: job_type.to_string(),
            payload_json: payload_json.to_string(),
            status: JOB_STATUS_QUEUED.to_string(),
            progress: 0.0,
            error_message: None,
            worker_pid: None,
            created_at: now_string(),
            started_at: None,
            completed_at: None,
        };

        self.db.with_conn(|conn| {
            diesel::insert_into(background_job::table)
                .values(&row)
                .execute(conn)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(id)
        })
    }

    fn claim_next(
        &self,
        job_type: Option<&str>,
    ) -> Result<Option<BackgroundJobRecord>, StoreError> {
        let worker_pid = std::process::id() as i32;
        let started_at = now_string();

        self.db.with_conn(|conn| {
            let mut query = background_job::table
                .filter(background_job::status.eq(JOB_STATUS_QUEUED))
                .order(background_job::created_at.asc())
                .into_boxed();

            if let Some(job_type) = job_type {
                query = query.filter(background_job::job_type.eq(job_type));
            }

            let candidate = query
                .select(background_job::id)
                .first::<String>(conn)
                .optional()
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;

            let Some(job_id) = candidate else {
                return Ok(None);
            };

            let updated = diesel::update(
                background_job::table
                    .filter(background_job::id.eq(&job_id))
                    .filter(background_job::status.eq(JOB_STATUS_QUEUED)),
            )
            .set((
                background_job::status.eq(ports::background_job::JOB_STATUS_RUNNING),
                background_job::started_at.eq(Some(started_at)),
                background_job::worker_pid.eq(Some(worker_pid)),
            ))
            .execute(conn)
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;

            if updated == 0 {
                return Ok(None);
            }

            background_job::table
                .filter(background_job::id.eq(job_id))
                .select(BackgroundJobRow::as_select())
                .first(conn)
                .optional()
                .map(|row| row.map(BackgroundJobRecord::from))
                .map_err(|error| StoreError::Unavailable(error.to_string()))
        })
    }

    fn mark_completed(&self, job_id: &str) -> Result<(), StoreError> {
        self.db.with_conn(|conn| {
            diesel::update(background_job::table.filter(background_job::id.eq(job_id)))
                .set((
                    background_job::status.eq(ports::background_job::JOB_STATUS_COMPLETED),
                    background_job::progress.eq(1.0_f32),
                    background_job::completed_at.eq(Some(now_string())),
                ))
                .execute(conn)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(())
        })
    }

    fn mark_failed(&self, job_id: &str, error_message: &str) -> Result<(), StoreError> {
        self.db.with_conn(|conn| {
            diesel::update(background_job::table.filter(background_job::id.eq(job_id)))
                .set((
                    background_job::status.eq(ports::background_job::JOB_STATUS_FAILED),
                    background_job::error_message.eq(Some(error_message.to_string())),
                    background_job::completed_at.eq(Some(now_string())),
                ))
                .execute(conn)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(())
        })
    }

    fn has_active_imap_sync(&self, account_id: &str) -> Result<bool, StoreError> {
        let needle = format!("\"account_id\":\"{account_id}\"");
        self.db.with_conn(|conn| {
            let count = background_job::table
                .filter(background_job::job_type.eq(ports::background_job::JOB_TYPE_IMAP_SYNC))
                .filter(
                    background_job::status
                        .eq(ports::background_job::JOB_STATUS_QUEUED)
                        .or(background_job::status.eq(ports::background_job::JOB_STATUS_RUNNING)),
                )
                .filter(background_job::payload_json.like(format!("%{needle}%")))
                .count()
                .get_result::<i64>(conn)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(count > 0)
        })
    }
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

impl From<BackgroundJobRow> for BackgroundJobRecord {
    fn from(row: BackgroundJobRow) -> Self {
        Self {
            id: row.id,
            job_type: row.job_type,
            payload_json: row.payload_json,
            status: row.status,
            progress: row.progress,
            error_message: row.error_message,
            worker_pid: row.worker_pid,
            created_at: row.created_at,
            started_at: row.started_at,
            completed_at: row.completed_at,
        }
    }
}
