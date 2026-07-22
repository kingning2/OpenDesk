//! Mail Tauri IPC commands.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use common::contracts::{
    MailIpcAccountListResponse, MailIpcAccountSaveRequest, MailIpcMessageListRequest,
    MailIpcMessageListResponse, MailIpcRecordInboundRequest, MailIpcRecordInboundResponse,
    MailIpcSendRequest, MailIpcSendResponse, MailIpcTemplateApplyRequest,
    MailIpcTemplateApplyResponse, MailIpcTemplateListResponse, MailIpcTemplateSaveRequest,
};
use mail::app::{
    ApplyMailTemplate, ListMailAccounts, ListMailMessages, ListMailTemplates, RecordInboundMail,
    SaveMailAccount, SaveMailTemplate, SendMail,
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
