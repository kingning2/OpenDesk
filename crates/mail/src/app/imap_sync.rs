//! IMAP inbox sync: fetch, persist, and IPC use cases.
//!
//! 作者：coisini
//! 创建时间：2026-07-22

use common::contracts::{
    MailIpcInboxUnmatchedListRequest, MailIpcInboxUnmatchedListResponse,
    MailIpcLinkInboundCustomerRequest, MailIpcLinkInboundCustomerResponse, MailIpcSyncNowRequest,
    MailIpcSyncNowResponse, MailIpcSyncStatusRequest, MailIpcSyncStatusResponse,
};
use mail_net::{fetch_messages_since, ImapEndpoint};
use ports::background_job::BackgroundJobStore;
use ports::customer::CustomerStore;
use ports::mail::{MailImapSyncStateRecord, MailStore, MailUnmatchedListFilter};

use crate::app::imap_inbound::{
    next_imap_sync_cursor, persist_imap_fetched_messages, IMAP_SYNC_MAX_FETCH,
};

use crate::app::mapper::{imap_sync_states_to_json, messages_to_json};

const DEFAULT_FOLDER: &str = "INBOX";

/// Run one IMAP fetch cycle for a single account and folder.
///
/// 作者：coisini
/// 创建时间：2026-08-01
pub struct RunImapAccountSync;

impl RunImapAccountSync {
    /// Connect to IMAP, fetch UID > `last_uid`, persist inbound rows, update cursor.
    ///
    /// 作者：coisini
    /// 创建时间：2026-08-01
    ///
    /// # 参数
    ///
    /// * `mail_store` - Mail accounts, messages, and sync state
    /// * `customer_store` - Customer lookup for auto-linking by email
    /// * `account_id` - Target mail account id
    /// * `folder` - IMAP folder (MVP: `INBOX`)
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - Sync finished (possibly zero new messages)
    /// * `Err(message)` - Connection, auth, or store failure
    pub fn execute<M: MailStore + ?Sized, C: CustomerStore + ?Sized>(
        mail_store: &M,
        customer_store: &C,
        account_id: &str,
        folder: &str,
    ) -> Result<(), String> {
        let folder = if folder.trim().is_empty() {
            DEFAULT_FOLDER
        } else {
            folder
        };

        let account = mail_store
            .get_account(account_id)
            .map_err(|error| error.to_string())?;
        let host = account
            .imap_host
            .clone()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "mail.imap_host_missing".to_string())?;
        let port = account.imap_port.unwrap_or(993).clamp(1, 65535) as u16;
        let use_tls = account.imap_use_tls.unwrap_or(port == 993);
        let password = mail_store
            .resolve_account_password(account_id)
            .map_err(|error| error.to_string())?;

        let state = mail_store
            .get_imap_sync_state(account_id, folder)
            .map_err(|error| error.to_string())?;

        let endpoint = ImapEndpoint {
            host,
            port,
            use_tls,
            username: account.username.clone(),
            password,
        };

        let last_uid = state.last_uid.max(0) as u32;
        tracing::info!(
            target: "lifecycle",
            %account_id,
            host = %endpoint.host,
            port = endpoint.port,
            folder,
            last_uid,
            user = %account.username,
            "imap account sync started"
        );
        match fetch_messages_since(&endpoint, folder, last_uid, IMAP_SYNC_MAX_FETCH) {
            Ok(result) => {
                let fetched_max_uid = result.fetched_max_uid;
                let search_max_uid = result.search_max_uid;
                let summary = persist_imap_fetched_messages(
                    mail_store,
                    customer_store,
                    account_id,
                    folder,
                    result.messages,
                )?;
                let cursor_uid = next_imap_sync_cursor(state.last_uid, fetched_max_uid);

                mail_store
                    .upsert_imap_sync_state(MailImapSyncStateRecord {
                        account_id: account_id.to_string(),
                        folder: folder.to_string(),
                        uidvalidity: state.uidvalidity,
                        highest_modseq: state.highest_modseq,
                        last_uid: cursor_uid,
                        last_sync_at: Some(now_string()),
                        last_error: None,
                        full_synced: true,
                    })
                    .map_err(|error| error.to_string())?;
                tracing::info!(
                    target: "lifecycle",
                    %account_id,
                    host = %endpoint.host,
                    folder,
                    inserted = summary.inserted,
                    skipped_unmatched = summary.skipped_unmatched,
                    fetched_max_uid = fetched_max_uid,
                    search_max_uid = search_max_uid,
                    last_uid = cursor_uid,
                    "imap account sync completed"
                );
                Ok(())
            }
            Err(message) => {
                tracing::warn!(
                    target: "lifecycle",
                    %account_id,
                    host = %endpoint.host,
                    folder,
                    error = %message,
                    "imap account sync failed"
                );
                mail_store
                    .upsert_imap_sync_state(MailImapSyncStateRecord {
                        account_id: account_id.to_string(),
                        folder: folder.to_string(),
                        uidvalidity: state.uidvalidity,
                        highest_modseq: state.highest_modseq,
                        last_uid: state.last_uid,
                        last_sync_at: state.last_sync_at,
                        last_error: Some(message.clone()),
                        full_synced: state.full_synced,
                    })
                    .map_err(|error| error.to_string())?;
                Err(message)
            }
        }
    }
}

/// Run IMAP sync for one or all enabled accounts.
///
/// 作者：coisini
/// 创建时间：2026-07-22
pub struct SyncMailNow;

impl SyncMailNow {
    /// Sync inbox inline for the requested account(s).
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-22
    pub fn execute<
        J: BackgroundJobStore + ?Sized,
        M: MailStore + ?Sized,
        C: CustomerStore + ?Sized,
    >(
        _job_store: &J,
        mail_store: &M,
        customer_store: &C,
        request: MailIpcSyncNowRequest,
    ) -> Result<MailIpcSyncNowResponse, String> {
        let accounts = if let Some(account_id) = request.account_id.as_deref() {
            let account = mail_store
                .get_account(account_id)
                .map_err(|error| error.to_string())?;
            if !account.imap_sync_enabled {
                return Err("mail.imap_sync_disabled".to_string());
            }
            vec![account]
        } else {
            mail_store
                .list_imap_sync_accounts()
                .map_err(|error| error.to_string())?
        };

        let mut synced = 0_i64;
        let mut last_error: Option<String> = None;
        for account in accounts {
            match RunImapAccountSync::execute(
                mail_store,
                customer_store,
                &account.id,
                DEFAULT_FOLDER,
            ) {
                Ok(()) => synced += 1,
                Err(message) => {
                    tracing::warn!(
                        target: "lifecycle",
                        account_id = %account.id,
                        error = %message,
                        "imap sync account failed"
                    );
                    last_error = Some(message);
                }
            }
        }

        if synced == 0 {
            if let Some(message) = last_error {
                return Err(message);
            }
            return Ok(MailIpcSyncNowResponse {
                job_ids_json: "[]".to_string(),
                enqueued: 0,
            });
        }

        Ok(MailIpcSyncNowResponse {
            job_ids_json: "[]".to_string(),
            enqueued: synced,
        })
    }
}

/// Read IMAP sync state for UI settings and polling.
///
/// 作者：coisini
/// 创建时间：2026-07-22
pub struct GetMailSyncStatus;

impl GetMailSyncStatus {
    /// Return sync cursor rows with active-job flags.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-22
    pub fn execute<J: BackgroundJobStore + ?Sized, M: MailStore + ?Sized>(
        job_store: &J,
        mail_store: &M,
        request: MailIpcSyncStatusRequest,
    ) -> Result<MailIpcSyncStatusResponse, String> {
        let states = mail_store
            .list_imap_sync_states(request.account_id.as_deref())
            .map_err(|error| error.to_string())?;

        let items_json = imap_sync_states_to_json(&states, |account_id| {
            job_store.has_active_imap_sync(account_id).unwrap_or(false)
        })?;

        Ok(MailIpcSyncStatusResponse {
            items_json,
            total: states.len() as i64,
        })
    }
}

/// List inbound messages waiting for customer association.
///
/// 作者：coisini
/// 创建时间：2026-07-22
pub struct ListUnmatchedInbound;

impl ListUnmatchedInbound {
    /// Return unmatched inbound rows for the mail workbench.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-22
    pub fn execute<M: MailStore + ?Sized>(
        mail_store: &M,
        request: MailIpcInboxUnmatchedListRequest,
    ) -> Result<MailIpcInboxUnmatchedListResponse, String> {
        let limit = request.limit.unwrap_or(50);
        let offset = request.offset.unwrap_or(0);
        let (items, total) = mail_store
            .list_unmatched_inbound(MailUnmatchedListFilter {
                account_id: request.account_id.clone(),
                limit,
                offset,
            })
            .map_err(|error| error.to_string())?;

        Ok(MailIpcInboxUnmatchedListResponse {
            messages_json: messages_to_json(&items)?,
            total,
        })
    }
}

/// Link one unmatched inbound message to a customer.
///
/// 作者：coisini
/// 创建时间：2026-07-22
pub struct LinkInboundCustomer;

impl LinkInboundCustomer {
    /// Associate one inbound message and append customer timeline.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-22
    pub fn execute<M: MailStore + ?Sized, C: CustomerStore + ?Sized>(
        mail_store: &M,
        customer_store: &C,
        request: MailIpcLinkInboundCustomerRequest,
    ) -> Result<MailIpcLinkInboundCustomerResponse, String> {
        customer_store
            .get(&request.customer_id)
            .map_err(|error| error.to_string())?;

        let record = mail_store
            .link_inbound_customer(&request.message_id, &request.customer_id)
            .map_err(|error| error.to_string())?;

        Ok(MailIpcLinkInboundCustomerResponse {
            message_id: record.id,
        })
    }
}

/// Run periodic IMAP sync for all enabled accounts (best-effort).
///
/// 作者：coisini
/// 创建时间：2026-07-22
pub struct ScheduleImapSync;

impl ScheduleImapSync {
    /// Sync every IMAP-enabled account inline.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-22
    pub fn execute<
        J: BackgroundJobStore + ?Sized,
        M: MailStore + ?Sized,
        C: CustomerStore + ?Sized,
    >(
        job_store: &J,
        mail_store: &M,
        customer_store: &C,
    ) -> Result<(), String> {
        let _ = SyncMailNow::execute(
            job_store,
            mail_store,
            customer_store,
            MailIpcSyncNowRequest { account_id: None },
        )?;
        Ok(())
    }
}

fn now_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
