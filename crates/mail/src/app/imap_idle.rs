//! IMAP IDLE — one Tokio task per enabled account, process mail as it arrives.
//!
//! 作者：coisini
//! 创建时间：2026-08-01

use std::sync::Arc;
use std::time::Duration;

use mail_net::{watch_inbox_idle, ImapEndpoint};
use ports::customer::CustomerStore;
use ports::mail::{MailImapSyncStateRecord, MailStore};

use super::imap_inbound::{
    next_imap_sync_cursor, persist_imap_fetched_messages, IMAP_SYNC_MAX_FETCH,
};

const DEFAULT_FOLDER: &str = "INBOX";
const IDLE_RETRY_DELAY_SECS: u64 = 5;

/// Callback invoked after an IDLE batch is persisted (account id).
pub type ImapIdlePersistHook = Arc<dyn Fn(&str) + Send + Sync>;

/// List enabled accounts and spawn one Tokio IDLE task per account.
///
/// 作者：coisini
/// 创建时间：2026-08-01
///
/// # 参数
///
/// - `mail_store` — mail persistence port
/// - `customer_store` — customer lookup port
/// - `on_persist` — optional hook after each IDLE batch (e.g. Tauri event emit)
pub fn spawn_imap_idle_watchers(
    mail_store: Arc<dyn MailStore>,
    customer_store: Arc<dyn CustomerStore>,
    on_persist: Option<ImapIdlePersistHook>,
) {
    let accounts = match mail_store.list_imap_sync_accounts() {
        Ok(accounts) => accounts,
        Err(error) => {
            tracing::warn!(target: "lifecycle", %error, "imap idle startup: list accounts failed");
            return;
        }
    };

    if accounts.is_empty() {
        tracing::info!(target: "lifecycle", "imap idle: no enabled accounts");
        return;
    }

    for account in accounts {
        let account_id = account.id;
        let mail_store = mail_store.clone();
        let customer_store = customer_store.clone();
        let on_persist = on_persist.clone();
        tokio::spawn(async move {
            tracing::info!(target: "lifecycle", %account_id, "imap idle watching");
            watch_account_idle(account_id, mail_store, customer_store, on_persist).await;
        });
    }
}

/// Keep one IMAP account connected with IDLE and persist inbound updates.
///
/// 作者：coisini
/// 创建时间：2026-08-01
///
/// # 参数
///
/// - `account_id` — mail account id
/// - `mail_store` — mail persistence port
/// - `customer_store` — customer lookup port
/// - `on_persist` — optional hook after each IDLE batch
pub async fn watch_account_idle(
    account_id: String,
    mail_store: Arc<dyn MailStore>,
    customer_store: Arc<dyn CustomerStore>,
    on_persist: Option<ImapIdlePersistHook>,
) {
    loop {
        let setup = prepare_account_watch(&account_id, mail_store.clone()).await;
        let (endpoint, folder, state) = match setup {
            Ok(values) => values,
            Err(message) => {
                let _ = mail_store.upsert_imap_sync_state(MailImapSyncStateRecord {
                    account_id: account_id.clone(),
                    folder: DEFAULT_FOLDER.to_string(),
                    uidvalidity: 0,
                    highest_modseq: "0".to_string(),
                    last_uid: 0,
                    last_sync_at: None,
                    last_error: Some(message.clone()),
                    full_synced: false,
                });
                tracing::warn!(target: "lifecycle", %account_id, %message, "imap idle setup failed");
                tokio::time::sleep(Duration::from_secs(IDLE_RETRY_DELAY_SECS)).await;
                continue;
            }
        };

        tracing::info!(
            target: "lifecycle",
            %account_id,
            host = %endpoint.host,
            folder = %folder,
            last_uid = state.last_uid,
            user = %endpoint.username,
            "imap idle connecting"
        );

        let result = tokio::task::spawn_blocking({
            let mail_store = mail_store.clone();
            let customer_store = customer_store.clone();
            let account_id = account_id.clone();
            let folder = folder.clone();
            let highest_modseq = state.highest_modseq.clone();
            let on_persist = on_persist.clone();
            move || {
                watch_inbox_idle(
                    &endpoint,
                    &folder,
                    state.last_uid.max(0) as u32,
                    IMAP_SYNC_MAX_FETCH,
                    |result| {
                        let fetched_max_uid = result.fetched_max_uid;
                        let search_max_uid = result.search_max_uid;
                        let summary = persist_imap_fetched_messages(
                            mail_store.as_ref(),
                            customer_store.as_ref(),
                            &account_id,
                            &folder,
                            result.messages,
                        )?;
                        let cursor_uid = next_imap_sync_cursor(state.last_uid, fetched_max_uid);

                        mail_store
                            .upsert_imap_sync_state(MailImapSyncStateRecord {
                                account_id: account_id.clone(),
                                folder: folder.clone(),
                                uidvalidity: state.uidvalidity,
                                highest_modseq: highest_modseq.clone(),
                                last_uid: cursor_uid,
                                last_sync_at: Some(now_string()),
                                last_error: None,
                                full_synced: true,
                            })
                            .map_err(|error| error.to_string())?;

                        tracing::info!(
                            target: "lifecycle",
                            %account_id,
                            inserted = summary.inserted,
                            skipped_unmatched = summary.skipped_unmatched,
                            fetched_max_uid = fetched_max_uid,
                            search_max_uid = search_max_uid,
                            last_uid = cursor_uid,
                            "imap idle batch persisted"
                        );

                        if summary.inserted > 0 {
                            if let Some(hook) = on_persist.as_ref() {
                                hook(&account_id);
                            }
                        }

                        Ok(fetched_max_uid)
                    },
                )
            }
        })
        .await;

        match result {
            Ok(Ok(())) => {}
            Ok(Err(message)) => {
                let _ = mail_store.upsert_imap_sync_state(MailImapSyncStateRecord {
                    account_id: account_id.clone(),
                    folder: folder.clone(),
                    uidvalidity: state.uidvalidity,
                    highest_modseq: state.highest_modseq.clone(),
                    last_uid: state.last_uid,
                    last_sync_at: state.last_sync_at.clone(),
                    last_error: Some(message.clone()),
                    full_synced: state.full_synced,
                });
                tracing::warn!(target: "lifecycle", %account_id, %message, "imap idle loop failed");
            }
            Err(error) => {
                tracing::warn!(target: "lifecycle", %account_id, %error, "imap idle join failed");
            }
        }

        tokio::time::sleep(Duration::from_secs(IDLE_RETRY_DELAY_SECS)).await;
    }
}

async fn prepare_account_watch(
    account_id: &str,
    mail_store: Arc<dyn MailStore>,
) -> Result<(ImapEndpoint, String, MailImapSyncStateRecord), String> {
    let account_id = account_id.to_string();
    tokio::task::spawn_blocking(move || {
        let account = mail_store
            .get_account(&account_id)
            .map_err(|error| error.to_string())?;
        let host = account
            .imap_host
            .clone()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "mail.imap_host_missing".to_string())?;
        let port = account.imap_port.unwrap_or(993).clamp(1, 65535) as u16;
        let use_tls = account.imap_use_tls.unwrap_or(port == 993);
        let password = mail_store
            .resolve_account_password(&account_id)
            .map_err(|error| error.to_string())?;
        let state = mail_store
            .get_imap_sync_state(&account_id, DEFAULT_FOLDER)
            .map_err(|error| error.to_string())?;

        Ok((
            ImapEndpoint {
                host,
                port,
                use_tls,
                username: account.username,
                password,
            },
            DEFAULT_FOLDER.to_string(),
            state,
        ))
    })
    .await
    .map_err(|error| error.to_string())?
}

fn now_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
