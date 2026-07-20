//! Handle `crawler_email_enrich` background jobs.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-20

use std::sync::Arc;
use std::time::Duration;

use crawler_enrich::{ChannelTarget, EnrichConfig, EnrichError};
use ports::background_job::{
    BackgroundJobRecord, CrawlerEmailEnrichPayload, EMAIL_STATUS_ENRICH_FAILED,
    EMAIL_STATUS_FOUND_PLAYWRIGHT, EMAIL_STATUS_NOT_FOUND,
};
use ports::crawler_channels::{CrawlerChannelStore, EmailEnrichResult};
use thiserror::Error;

use crate::job_runner::now_string;

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("invalid payload: {0}")]
    InvalidPayload(String),
    #[error("store error: {0}")]
    Store(#[from] ports::repository::StoreError),
    #[error("enrich error: {0}")]
    Enrich(#[from] EnrichError),
}

/// Execute one crawler email enrichment job.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-20
pub async fn handle(
    job: &BackgroundJobRecord,
    channel_store: Arc<dyn CrawlerChannelStore>,
) -> Result<(), HandlerError> {
    let payload: CrawlerEmailEnrichPayload = serde_json::from_str(&job.payload_json)
        .map_err(|error| HandlerError::InvalidPayload(error.to_string()))?;

    channel_store.mark_enriching(&payload.crawler_job_id, &payload.channel_id)?;

    let config = enrich_config_from_env();
    let target = ChannelTarget {
        channel_id: payload.channel_id.clone(),
        custom_url: payload.custom_url.clone(),
    };

    throttle_between_jobs().await;

    let enrich_result = match crawler_enrich::fetch_email_about_page(&target, &config).await {
        Ok(Some(email)) => EmailEnrichResult {
            email: Some(email),
            email_status: EMAIL_STATUS_FOUND_PLAYWRIGHT.to_string(),
            enrich_error: None,
        },
        Ok(None) => EmailEnrichResult {
            email: None,
            email_status: EMAIL_STATUS_NOT_FOUND.to_string(),
            enrich_error: None,
        },
        Err(error) => EmailEnrichResult {
            email: None,
            email_status: EMAIL_STATUS_ENRICH_FAILED.to_string(),
            enrich_error: Some(error.to_string()),
        },
    };

    channel_store.apply_enrich_result(
        &payload.crawler_job_id,
        &payload.channel_id,
        &enrich_result,
        &now_string(),
    )?;

    Ok(())
}

async fn throttle_between_jobs() {
    let delay_ms = std::env::var("CRAWLER_ENRICH_DELAY_MS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(2000);
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
}

fn enrich_config_from_env() -> EnrichConfig {
    let mut config = EnrichConfig::default();
    if let Ok(path) = std::env::var("CRAWLER_ENRICH_CHROME_PATH") {
        if !path.trim().is_empty() {
            config.chrome_path = Some(path.into());
        }
    }
    if let Ok(dir) = std::env::var("CRAWLER_ENRICH_TEMPLATE_DIR") {
        if !dir.trim().is_empty() {
            config.template_dir = dir.into();
        }
    }
    config
}
