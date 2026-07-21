//! Claim and dispatch `background_job` rows.
//!
//! 作者：coisini
//! 创建时间：2026-07-20

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use ports::background_job::{
    BackgroundJobStore, JOB_STATUS_COMPLETED, JOB_TYPE_CRAWLER_EMAIL_ENRICH,
};
use ports::crawler_channels::CrawlerChannelStore;
use thiserror::Error;

use crate::handlers::crawler_email_enrich;

/// Poll loop orchestrator for Worker jobs.
pub struct JobRunner {
    job_store: Arc<dyn BackgroundJobStore>,
    channel_store: Arc<dyn CrawlerChannelStore>,
}

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("store error: {0}")]
    Store(#[from] ports::repository::StoreError),
}

impl JobRunner {
    /// Create a runner bound to shared SQLite stores.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-20
    pub fn new(
        job_store: Arc<dyn BackgroundJobStore>,
        channel_store: Arc<dyn CrawlerChannelStore>,
    ) -> Self {
        Self {
            job_store,
            channel_store,
        }
    }

    /// Claim and execute at most one queued job.
    ///
    /// # 返回值
    /// - `Ok(true)` — a job was claimed (success or failure already persisted)
    /// - `Ok(false)` — queue empty
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-20
    pub async fn poll_once(&self) -> Result<bool, RunnerError> {
        let Some(job) = self.job_store.claim_next(None)? else {
            return Ok(false);
        };

        tracing::info!(job_id = %job.id, job_type = %job.job_type, "claimed background job");

        let result: Result<(), String> = match job.job_type.as_str() {
            JOB_TYPE_CRAWLER_EMAIL_ENRICH => {
                crawler_email_enrich::handle(&job, self.channel_store.clone())
                    .await
                    .map_err(|error| error.to_string())
            }
            other => Err(format!("unsupported job_type={other}")),
        };

        match result {
            Ok(()) => {
                self.job_store.mark_completed(&job.id)?;
                tracing::info!(job_id = %job.id, status = JOB_STATUS_COMPLETED, "job completed");
            }
            Err(message) => {
                self.job_store.mark_failed(&job.id, &message)?;
                tracing::warn!(job_id = %job.id, %message, "job failed");
            }
        }

        Ok(true)
    }
}

/// Unix timestamp string used for enrichment timestamps.
pub(crate) fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
