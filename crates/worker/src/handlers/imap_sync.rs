//! Handle `imap_sync` background jobs.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-22

use std::sync::Arc;

use std::time::Duration;

use mail_net::{fetch_messages_since, watch_inbox_idle, ImapEndpoint};
use ports::background_job::{BackgroundJobRecord, ImapSyncPayload};
use ports::customer::CustomerStore;
use ports::mail::{MailImapInboundWriteInput, MailImapSyncStateRecord, MailStore};
use thiserror::Error;

use crate::job_runner::now_string;

const DEFAULT_FOLDER: &str = "INBOX";
const IDLE_RETRY_DELAY_SECS: u64 = 5;

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("invalid payload: {0}")]
    InvalidPayload(String),
    #[error("store error: {0}")]
    Store(#[from] ports::repository::StoreError),
    #[error("imap error: {0}")]
    Imap(String),
}

/// Execute one IMAP inbox sync job.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub async fn handle(
    job: &BackgroundJobRecord,
    mail_store: Arc<dyn MailStore>,
    customer_store: Arc<dyn CustomerStore>,
) -> Result<(), HandlerError> {
    let payload: ImapSyncPayload = serde_json::from_str(&job.payload_json)
        .map_err(|error| HandlerError::InvalidPayload(error.to_string()))?;

    let folder = if payload.folder.trim().is_empty() {
        DEFAULT_FOLDER.to_string()
    } else {
        payload.folder.clone()
    };

    let account = mail_store
        .get_account(&payload.account_id)
        .map_err(HandlerError::Store)?;
    let host = account
        .imap_host
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| HandlerError::Imap("mail.imap_host_missing".to_string()))?;
    let port = account.imap_port.unwrap_or(993).clamp(1, 65535) as u16;
    let use_tls = account.imap_use_tls.unwrap_or(port == 993);
    let password = mail_store
        .resolve_account_password(&payload.account_id)
        .map_err(HandlerError::Store)?;

    let state = mail_store
        .get_imap_sync_state(&payload.account_id, &folder)
        .map_err(HandlerError::Store)?;

    let endpoint = ImapEndpoint {
        host,
        port,
        use_tls,
        username: account.username.clone(),
        password,
    };

    let fetch_result = tokio::task::spawn_blocking({
        let endpoint = endpoint.clone();
        let folder = folder.clone();
        let last_uid = state.last_uid.max(0) as u32;
        move || fetch_messages_since(&endpoint, &folder, last_uid)
    })
    .await
    .map_err(|error| HandlerError::Imap(error.to_string()))?;

    match fetch_result {
        Ok(messages) => {
            let mut max_uid = state.last_uid;
            for message in messages {
                max_uid = max_uid.max(message.uid as i64);
                let customer_id = customer_store
                    .find_by_email(&message.from_address)
                    .map_err(HandlerError::Store)?
                    .map(|record| record.id);

                let should_store = should_store_inbound(
                    mail_store.as_ref(),
                    &payload.account_id,
                    &message.from_address,
                    message.in_reply_to.as_deref(),
                    message.references.as_deref(),
                )
                .map_err(HandlerError::Imap)?;
                if !should_store {
                    continue;
                }

                let _ = mail_store.insert_imap_inbound_if_new(MailImapInboundWriteInput {
                    account_id: payload.account_id.clone(),
                    customer_id,
                    from_address: message.from_address,
                    from_name: message.from_name,
                    subject: message.subject,
                    body_text: message.body_text,
                    body_html: message.body_html,
                    received_at: message.received_at,
                    imap_uid: message.uid as i64,
                    imap_folder: folder.clone(),
                    rfc_message_id: message.rfc_message_id,
                    in_reply_to: message.in_reply_to,
                    references: message.references,
                })?;
            }

            mail_store
                .upsert_imap_sync_state(MailImapSyncStateRecord {
                    account_id: payload.account_id.clone(),
                    folder: folder.clone(),
                    uidvalidity: state.uidvalidity,
                    highest_modseq: state.highest_modseq,
                    last_uid: max_uid,
                    last_sync_at: Some(now_string()),
                    last_error: None,
                    full_synced: true,
                })
                .map_err(HandlerError::Store)?;
            Ok(())
        }
        Err(message) => {
            mail_store
                .upsert_imap_sync_state(MailImapSyncStateRecord {
                    account_id: payload.account_id.clone(),
                    folder,
                    uidvalidity: state.uidvalidity,
                    highest_modseq: state.highest_modseq,
                    last_uid: state.last_uid,
                    last_sync_at: state.last_sync_at,
                    last_error: Some(message.clone()),
                    full_synced: state.full_synced,
                })
                .map_err(HandlerError::Store)?;
            Err(HandlerError::Imap(message))
        }
    }
}

/// Keep one IMAP account connected with IDLE and persist inbound updates.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub async fn watch_account_idle(
    account_id: String,
    mail_store: Arc<dyn MailStore>,
    customer_store: Arc<dyn CustomerStore>,
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
                tracing::warn!(%account_id, %message, "imap idle setup failed");
                tokio::time::sleep(Duration::from_secs(IDLE_RETRY_DELAY_SECS)).await;
                continue;
            }
        };

        let result = tokio::task::spawn_blocking({
            let mail_store = mail_store.clone();
            let customer_store = customer_store.clone();
            let account_id = account_id.clone();
            let folder = folder.clone();
            let highest_modseq = state.highest_modseq.clone();
            move || {
                watch_inbox_idle(
                    &endpoint,
                    &folder,
                    state.last_uid.max(0) as u32,
                    |messages| {
                        let mut max_uid = state.last_uid;
                        for message in messages {
                            max_uid = max_uid.max(message.uid as i64);
                            let customer_id = customer_store
                                .find_by_email(&message.from_address)
                                .map_err(|error| error.to_string())?
                                .map(|record| record.id);

                            let should_store = should_store_inbound(
                                mail_store.as_ref(),
                                &account_id,
                                &message.from_address,
                                message.in_reply_to.as_deref(),
                                message.references.as_deref(),
                            )?;
                            if !should_store {
                                continue;
                            }

                            mail_store
                                .insert_imap_inbound_if_new(MailImapInboundWriteInput {
                                    account_id: account_id.clone(),
                                    customer_id,
                                    from_address: message.from_address,
                                    from_name: message.from_name,
                                    subject: message.subject,
                                    body_text: message.body_text,
                                    body_html: message.body_html,
                                    received_at: message.received_at,
                                    imap_uid: message.uid as i64,
                                    imap_folder: folder.clone(),
                                    rfc_message_id: message.rfc_message_id,
                                    in_reply_to: message.in_reply_to,
                                    references: message.references,
                                })
                                .map_err(|error| error.to_string())?;
                        }

                        mail_store
                            .upsert_imap_sync_state(MailImapSyncStateRecord {
                                account_id: account_id.clone(),
                                folder: folder.clone(),
                                uidvalidity: state.uidvalidity,
                                highest_modseq: highest_modseq.clone(),
                                last_uid: max_uid,
                                last_sync_at: Some(now_string()),
                                last_error: None,
                                full_synced: true,
                            })
                            .map_err(|error| error.to_string())?;

                        Ok(max_uid.max(state.last_uid) as u32)
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
                tracing::warn!(%account_id, %message, "imap idle loop failed");
            }
            Err(error) => {
                tracing::warn!(%account_id, %error, "imap idle join failed");
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

fn should_store_inbound(
    mail_store: &dyn MailStore,
    account_id: &str,
    from_address: &str,
    in_reply_to: Option<&str>,
    references: Option<&str>,
) -> Result<bool, String> {
    if let Some(reply_id) = in_reply_to.map(str::trim).filter(|value| !value.is_empty()) {
        if mail_store
            .is_outbound_message_id(account_id, reply_id)
            .map_err(|error| error.to_string())?
        {
            return Ok(true);
        }
    }

    if let Some(references) = references.map(str::trim).filter(|value| !value.is_empty()) {
        for token in references.split_whitespace() {
            if mail_store
                .is_outbound_message_id(account_id, token)
                .map_err(|error| error.to_string())?
            {
                return Ok(true);
            }
        }
    }

    mail_store
        .has_outbound_to_address(account_id, from_address)
        .map_err(|error| error.to_string())
}
