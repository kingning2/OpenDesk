//! IMAP sync scheduling and status use cases.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-22

use common::contracts::{
    MailIpcInboxUnmatchedListRequest, MailIpcInboxUnmatchedListResponse,
    MailIpcLinkInboundCustomerRequest, MailIpcLinkInboundCustomerResponse, MailIpcSyncNowRequest,
    MailIpcSyncNowResponse, MailIpcSyncStatusRequest, MailIpcSyncStatusResponse,
};
use ports::background_job::{BackgroundJobStore, ImapSyncPayload, JOB_TYPE_IMAP_SYNC};
use ports::customer::CustomerStore;
use ports::mail::{MailStore, MailUnmatchedListFilter};

use crate::app::mapper::{imap_sync_states_to_json, messages_to_json};

const DEFAULT_FOLDER: &str = "INBOX";

/// Enqueue one or more IMAP sync background jobs.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub struct SyncMailNow;

impl SyncMailNow {
    /// Queue IMAP sync jobs for one account or all enabled accounts.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-22
    pub fn execute<J: BackgroundJobStore + ?Sized, M: MailStore + ?Sized>(
        job_store: &J,
        mail_store: &M,
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

        let mut job_ids = Vec::new();
        for account in accounts {
            if job_store
                .has_active_imap_sync(&account.id)
                .map_err(|error| error.to_string())?
            {
                continue;
            }
            let payload = ImapSyncPayload {
                account_id: account.id.clone(),
                folder: DEFAULT_FOLDER.to_string(),
            };
            let payload_json =
                serde_json::to_string(&payload).map_err(|error| error.to_string())?;
            let job_id = job_store
                .enqueue(JOB_TYPE_IMAP_SYNC, &payload_json)
                .map_err(|error| error.to_string())?;
            job_ids.push(job_id);
        }

        let enqueued = job_ids.len() as i64;
        let job_ids_json = serde_json::to_string(&job_ids).map_err(|error| error.to_string())?;
        Ok(MailIpcSyncNowResponse {
            job_ids_json,
            enqueued,
        })
    }
}

/// Read IMAP sync state for UI settings and polling.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub struct GetMailSyncStatus;

impl GetMailSyncStatus {
    /// Return sync cursor rows with active-job flags.
    ///
    /// 作者：Xiaoman
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
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub struct ListUnmatchedInbound;

impl ListUnmatchedInbound {
    /// Return unmatched inbound rows for the mail workbench.
    ///
    /// 作者：Xiaoman
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
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub struct LinkInboundCustomer;

impl LinkInboundCustomer {
    /// Associate one inbound message and append customer timeline.
    ///
    /// 作者：Xiaoman
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

/// Enqueue periodic IMAP sync for all enabled accounts (best-effort).
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub struct ScheduleImapSync;

impl ScheduleImapSync {
    /// Queue sync jobs when none are already active for an account.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-22
    pub fn execute<J: BackgroundJobStore + ?Sized, M: MailStore + ?Sized>(
        job_store: &J,
        mail_store: &M,
    ) -> Result<(), String> {
        let _ = SyncMailNow::execute(
            job_store,
            mail_store,
            MailIpcSyncNowRequest { account_id: None },
        )?;
        Ok(())
    }
}
