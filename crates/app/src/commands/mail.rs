//! Mail Tauri IPC commands.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use common::contracts::{
    MailIpcAccountListResponse, MailIpcAccountSaveRequest, MailIpcInboxUnmatchedListRequest,
    MailIpcInboxUnmatchedListResponse, MailIpcLinkInboundCustomerRequest,
    MailIpcLinkInboundCustomerResponse, MailIpcMessageListRequest, MailIpcMessageListResponse,
    MailIpcRecordInboundRequest, MailIpcRecordInboundResponse, MailIpcSendRequest,
    MailIpcSendResponse, MailIpcSyncNowRequest, MailIpcSyncNowResponse, MailIpcSyncStatusRequest,
    MailIpcSyncStatusResponse, MailIpcTemplateApplyRequest, MailIpcTemplateApplyResponse,
    MailIpcTemplateListResponse, MailIpcTemplateSaveRequest,
};
use mail::app::{
    ApplyMailTemplate, GetMailSyncStatus, LinkInboundCustomer, ListMailAccounts, ListMailMessages,
    ListMailTemplates, ListUnmatchedInbound, RecordInboundMail, SaveMailAccount, SaveMailTemplate,
    SendMail, SyncMailNow,
};

use crate::state::AppState;

/// List mail templates.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[tauri::command]
pub async fn mail_template_list(
    state: tauri::State<'_, AppState>,
) -> Result<MailIpcTemplateListResponse, String> {
    let store = state.mail_store.clone();
    tauri::async_runtime::spawn_blocking(move || ListMailTemplates::execute(store.as_ref()))
        .await
        .map_err(|error| error.to_string())?
}

/// Create or update a custom mail template.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[tauri::command]
pub async fn mail_template_save(
    state: tauri::State<'_, AppState>,
    request: MailIpcTemplateSaveRequest,
) -> Result<MailIpcTemplateListResponse, String> {
    let store = state.mail_store.clone();
    tauri::async_runtime::spawn_blocking(move || SaveMailTemplate::execute(store.as_ref(), request))
        .await
        .map_err(|error| error.to_string())?
}

/// Apply one template to one customer.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[tauri::command]
pub async fn mail_template_apply(
    state: tauri::State<'_, AppState>,
    request: MailIpcTemplateApplyRequest,
) -> Result<MailIpcTemplateApplyResponse, String> {
    let mail_store = state.mail_store.clone();
    let customer_store = state.customer_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        ApplyMailTemplate::execute(mail_store.as_ref(), customer_store.as_ref(), request)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// List saved mail accounts.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[tauri::command]
pub async fn mail_account_list(
    state: tauri::State<'_, AppState>,
) -> Result<MailIpcAccountListResponse, String> {
    let store = state.mail_store.clone();
    tauri::async_runtime::spawn_blocking(move || ListMailAccounts::execute(store.as_ref()))
        .await
        .map_err(|error| error.to_string())?
}

/// Save one mail account.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[tauri::command]
pub async fn mail_account_save(
    state: tauri::State<'_, AppState>,
    request: MailIpcAccountSaveRequest,
) -> Result<MailIpcAccountListResponse, String> {
    let store = state.mail_store.clone();
    tauri::async_runtime::spawn_blocking(move || SaveMailAccount::execute(store.as_ref(), request))
        .await
        .map_err(|error| error.to_string())?
}

/// List local inbox/sent messages.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[tauri::command]
pub async fn mail_message_list(
    state: tauri::State<'_, AppState>,
    request: MailIpcMessageListRequest,
) -> Result<MailIpcMessageListResponse, String> {
    let store = state.mail_store.clone();
    tauri::async_runtime::spawn_blocking(move || ListMailMessages::execute(store.as_ref(), request))
        .await
        .map_err(|error| error.to_string())?
}

/// Send one outbound email via SMTP and persist the result.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[tauri::command]
pub async fn mail_send(
    state: tauri::State<'_, AppState>,
    request: MailIpcSendRequest,
) -> Result<MailIpcSendResponse, String> {
    let mail_store = state.mail_store.clone();
    let customer_store = state.customer_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        SendMail::execute(mail_store.as_ref(), customer_store.as_ref(), request)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Record one inbound email manually.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[tauri::command]
pub async fn mail_record_inbound(
    state: tauri::State<'_, AppState>,
    request: MailIpcRecordInboundRequest,
) -> Result<MailIpcRecordInboundResponse, String> {
    let mail_store = state.mail_store.clone();
    let customer_store = state.customer_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        RecordInboundMail::execute(mail_store.as_ref(), customer_store.as_ref(), request)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Enqueue IMAP sync background jobs.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[tauri::command]
pub async fn mail_sync_now(
    state: tauri::State<'_, AppState>,
    request: MailIpcSyncNowRequest,
) -> Result<MailIpcSyncNowResponse, String> {
    let job_store = state.job_store.clone();
    let mail_store = state.mail_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        SyncMailNow::execute(job_store.as_ref(), mail_store.as_ref(), request)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Read IMAP sync status for mail settings UI.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[tauri::command]
pub async fn mail_sync_status(
    state: tauri::State<'_, AppState>,
    request: MailIpcSyncStatusRequest,
) -> Result<MailIpcSyncStatusResponse, String> {
    let job_store = state.job_store.clone();
    let mail_store = state.mail_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        GetMailSyncStatus::execute(job_store.as_ref(), mail_store.as_ref(), request)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// List inbound messages without a linked customer.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[tauri::command]
pub async fn mail_inbox_unmatched_list(
    state: tauri::State<'_, AppState>,
    request: MailIpcInboxUnmatchedListRequest,
) -> Result<MailIpcInboxUnmatchedListResponse, String> {
    let mail_store = state.mail_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        ListUnmatchedInbound::execute(mail_store.as_ref(), request)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Link one unmatched inbound message to a customer.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[tauri::command]
pub async fn mail_link_inbound_customer(
    state: tauri::State<'_, AppState>,
    request: MailIpcLinkInboundCustomerRequest,
) -> Result<MailIpcLinkInboundCustomerResponse, String> {
    let mail_store = state.mail_store.clone();
    let customer_store = state.customer_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        LinkInboundCustomer::execute(mail_store.as_ref(), customer_store.as_ref(), request)
    })
    .await
    .map_err(|error| error.to_string())?
}
