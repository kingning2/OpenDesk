//! `opendesk-worker` binary — polls `background_job` and runs heavy tasks.
//!
//! 作者：coisini
//! 创建时间：2026-07-20

mod handlers;
mod job_runner;
mod paths;

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use handlers::imap_sync;
use job_runner::JobRunner;
use paths::{crawler_db_path, opendesk_db_path};
use ports::customer::CustomerStore;
use ports::mail::MailStore;
use storage::background_job::SqliteBackgroundJobStore;
use storage::crawler_channels::SqliteCrawlerChannelStore;
use storage::customer::SqliteCustomerStore;
use storage::mail::SqliteMailStore;
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
    let opendesk_db =
        storage::opendesk_db::OpendeskDb::open(&opendesk_path).expect("open opendesk.db");
    let mail_store = Arc::new(SqliteMailStore::new(opendesk_db.clone()));
    let customer_store = Arc::new(SqliteCustomerStore::new(opendesk_db));

    spawn_imap_idle_supervisor(mail_store.clone(), customer_store.clone());

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

fn spawn_imap_idle_supervisor(
    mail_store: Arc<dyn MailStore>,
    customer_store: Arc<dyn CustomerStore>,
) {
    let active_accounts = Arc::new(std::sync::Mutex::new(HashSet::<String>::new()));
    spawn_imap_idle_supervisor_loop(mail_store, customer_store, active_accounts);
}

fn spawn_imap_idle_supervisor_loop(
    mail_store: Arc<dyn MailStore>,
    customer_store: Arc<dyn CustomerStore>,
    active_accounts: Arc<std::sync::Mutex<HashSet<String>>>,
) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(15));
        loop {
            ticker.tick().await;
            match mail_store.list_imap_sync_accounts() {
                Ok(accounts) => {
                    for account in accounts {
                        let mut guard = match active_accounts.lock() {
                            Ok(guard) => guard,
                            Err(_) => continue,
                        };
                        if !guard.insert(account.id.clone()) {
                            continue;
                        }
                        drop(guard);

                        let mail_store = mail_store.clone();
                        let customer_store = customer_store.clone();
                        let active_accounts = active_accounts.clone();
                        let account_id = account.id.clone();
                        tokio::spawn(async move {
                            imap_sync::watch_account_idle(
                                account_id.clone(),
                                mail_store,
                                customer_store,
                            )
                            .await;
                            if let Ok(mut guard) = active_accounts.lock() {
                                guard.remove(&account_id);
                            }
                        });
                    }
                }
                Err(error) => tracing::warn!(%error, "list imap sync accounts failed"),
            }
        }
    });
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(
            "info,opendesk_worker=info,crawler_enrich=info",
        ))
        .try_init();
}
