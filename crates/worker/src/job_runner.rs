//! Claim and dispatch `background_job` rows.
//!
//! 作者：coisini
//! 创建时间：2026-07-20

use std::sync::Arc;

use agent::embedding::Embedder;
use ports::background_job::{
    BackgroundJobStore, JOB_STATUS_COMPLETED, JOB_TYPE_CRAWLER_EMAIL_ENRICH, JOB_TYPE_IMAP_SYNC,
    JOB_TYPE_KNOWLEDGE_IMPORT,
};
use ports::crawler_channels::CrawlerChannelStore;
use ports::customer::CustomerStore;
use ports::knowledge::KnowledgeStore;
use ports::mail::MailStore;
use thiserror::Error;

use crate::handlers::{crawler_email_enrich, imap_sync, knowledge_import};

/// Poll loop orchestrator for Worker jobs.
pub struct JobRunner {
    job_store: Arc<dyn BackgroundJobStore>,
    channel_store: Arc<dyn CrawlerChannelStore>,
    mail_store: Arc<dyn MailStore>,
    customer_store: Arc<dyn CustomerStore>,
    knowledge_store: Arc<dyn KnowledgeStore>,
    embedder: Arc<dyn Embedder>,
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
        mail_store: Arc<dyn MailStore>,
        customer_store: Arc<dyn CustomerStore>,
        knowledge_store: Arc<dyn KnowledgeStore>,
        embedder: Arc<dyn Embedder>,
    ) -> Self {
        Self {
            job_store,
            channel_store,
            mail_store,
            customer_store,
            knowledge_store,
            embedder,
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
        let job = self
            .job_store
            .claim_next(Some(JOB_TYPE_IMAP_SYNC))?
            .or(self.job_store.claim_next(None)?);
        let Some(job) = job else {
            return Ok(false);
        };

        tracing::info!(job_id = %job.id, job_type = %job.job_type, "claimed background job");

        let result: Result<(), String> = match job.job_type.as_str() {
            JOB_TYPE_CRAWLER_EMAIL_ENRICH => {
                crawler_email_enrich::handle(&job, self.channel_store.clone())
                    .await
                    .map_err(|error| error.to_string())
            }
            JOB_TYPE_IMAP_SYNC => {
                imap_sync::handle(&job, self.mail_store.clone(), self.customer_store.clone())
                    .await
                    .map_err(|error| error.to_string())
            }
            JOB_TYPE_KNOWLEDGE_IMPORT => {
                knowledge_import::handle(&job, self.knowledge_store.clone(), self.embedder.clone())
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
