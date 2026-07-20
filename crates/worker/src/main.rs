//! `opendesk-worker` binary — polls `background_job` and runs heavy tasks.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-20

mod handlers;
mod job_runner;
mod paths;

use std::sync::Arc;
use std::time::Duration;

use job_runner::JobRunner;
use paths::{crawler_db_path, opendesk_db_path};
use storage::background_job::SqliteBackgroundJobStore;
use storage::crawler_channels::SqliteCrawlerChannelStore;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    init_tracing();

    let opendesk_path = opendesk_db_path();
    let crawler_path = crawler_db_path();
    tracing::info!(
        opendesk_db = %opendesk_path.display(),
        crawler_db = %crawler_path.display(),
        "opendesk-worker starting"
    );

    let job_store =
        Arc::new(SqliteBackgroundJobStore::open(&opendesk_path).expect("open opendesk.db"));
    let channel_store =
        Arc::new(SqliteCrawlerChannelStore::open(&crawler_path).expect("open crawler.db"));

    let runner = JobRunner::new(job_store, channel_store);
    let poll_ms = std::env::var("OPENDESK_WORKER_POLL_MS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1000);

    loop {
        match runner.poll_once().await {
            Ok(true) => continue,
            Ok(false) => tokio::time::sleep(Duration::from_millis(poll_ms)).await,
            Err(error) => {
                tracing::error!(%error, "worker poll failed");
                tokio::time::sleep(Duration::from_millis(poll_ms)).await;
            }
        }
    }
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(
            "info,opendesk_worker=info,crawler_enrich=info",
        ))
        .try_init();
}
