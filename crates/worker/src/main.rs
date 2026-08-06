//! `opendesk-worker` binary — polls `background_job` and runs heavy tasks.
//!
//! 作者：coisini
//! 创建时间：2026-07-20

mod handlers;
mod job_runner;
mod paths;

use std::sync::Arc;
use std::time::Duration;

use fs2::FileExt;
use job_runner::JobRunner;
use mail::app::spawn_imap_idle_watchers;
use paths::{crawler_db_path, opendesk_db_path, worker_lock_path};
use storage::background_job::SqliteBackgroundJobStore;
use storage::crawler_channels::SqliteCrawlerChannelStore;
use storage::customer::SqliteCustomerStore;
use storage::mail::SqliteMailStore;
#[tokio::main]
async fn main() {
    kernel::logging::init_tracing("opendesk-worker");

    // 单实例锁：主进程会自动拉起 worker，手动启动也不会重复运行。
    let lock_path = worker_lock_path();
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
        .expect("open worker lock file");
    match lock_file.try_lock_exclusive() {
        Ok(()) => {}
        Err(error)
            if error.kind() == std::io::ErrorKind::WouldBlock
                || error.raw_os_error() == Some(33) =>
        {
            // Windows 上锁被持有返回 ERROR_LOCK_VIOLATION(33)，Unix 为 WouldBlock。
            tracing::info!(
                target: "lifecycle",
                ?lock_path,
                "another opendesk-worker instance is running; exiting"
            );
            std::process::exit(0);
        }
        Err(error) => {
            tracing::warn!(
                target: "lifecycle",
                %error,
                ?lock_path,
                "worker lock check failed; continuing"
            );
        }
    }
    // 持有文件到进程退出，文件锁随之自动释放。
    let _worker_lock = lock_file;

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
