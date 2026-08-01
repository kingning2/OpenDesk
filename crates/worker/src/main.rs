//! `opendesk-worker` binary — polls `background_job` and runs heavy tasks.
//!
//! 作者：coisini
//! 创建时间：2026-07-20

mod handlers;
mod job_runner;
mod paths;

use std::sync::Arc;
use std::time::Duration;

use job_runner::JobRunner;
use mail::app::spawn_imap_idle_watchers;
use paths::{crawler_db_path, opendesk_db_path};
use storage::background_job::SqliteBackgroundJobStore;
use storage::crawler_channels::SqliteCrawlerChannelStore;
use storage::customer::SqliteCustomerStore;
use storage::mail::SqliteMailStore;
#[tokio::main]
async fn main() {
    kernel::logging::init_tracing("opendesk-worker");

    let opendesk_path = opendesk_db_path();
    let crawler_path = crawler_db_path();
    tracing::info!(
        target: "lifecycle",
        opendesk_db = %opendesk_path.display(),
        crawler_db = %crawler_path.display(),
        log_dir = %kernel::logging::log_dir().display(),
        "opendesk-worker starting"
    );

    let job_store =
        Arc::new(SqliteBackgroundJobStore::open(&opendesk_path).expect("open opendesk.db"));
    let channel_store =
        Arc::new(SqliteCrawlerChannelStore::open(&crawler_path).expect("open crawler.db"));
    let opendesk_db =
        storage::opendesk_db::OpendeskDb::open(&opendesk_path).expect("open opendesk.db");
    let mail_store = Arc::new(SqliteMailStore::new(opendesk_db.clone()));
    let customer_store = Arc::new(SqliteCustomerStore::new(opendesk_db));

    spawn_imap_idle_watchers(mail_store.clone(), customer_store.clone(), None);

    let runner = JobRunner::new(job_store, channel_store, mail_store, customer_store);
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
