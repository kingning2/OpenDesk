//! 渠道通用 Tauri commands（平台无关）。
//!
//! 闲鱼专属命令（历史 / 商品卡 / 渠道扫码）已迁往
//! `platforms::xianyu::ipc::chat`；此处仅留状态 / 连接 / 发送。

use common::contracts::{
    ChannelConversation, ChannelIpcConnectResponse, ChannelIpcDisconnectResponse,
    ChannelIpcSendRequest, ChannelIpcSendResponse, ChannelIpcStateRequest, ChannelIpcStateResponse,
};
use std::sync::Arc;
use tauri::State;

use crate::shared::channel::coordinator::ChannelCoordinator;
use crate::shared::channel::dispatcher::ChannelDispatcher;
use crate::shared::channel::ChannelRepo;
use crate::shared::ipc::IpcResponse;
use common::DingDaResult;

/// 读取渠道全量状态（账号/会话/消息/设置）。

#[tauri::command]
pub async fn channel_state_get(
    repo: State<'_, Arc<ChannelRepo>>,
) -> DingDaResult<IpcResponse<ChannelIpcStateResponse>> {
    let accounts = repo.list_accounts().map_err(|error| error.to_string())?;
    let conversations = repo
        .list_conversations()
        .map_err(|error| error.to_string())?;
    let messages = repo
        .list_all_messages()
        .map_err(|error| error.to_string())?;
    let settings = repo.get_settings().map_err(|error| error.to_string())?;
    Ok(IpcResponse::ok(ChannelIpcStateResponse {
        accounts,
        conversations,
        messages,
        settings,
    }))
}

/// 保存渠道配置（账号 + 设置）。

#[tauri::command]
pub async fn channel_state_set(
    repo: State<'_, Arc<ChannelRepo>>,
    request: ChannelIpcStateRequest,
) -> DingDaResult<IpcResponse<ChannelIpcStateResponse>> {
    for account in &request.accounts {
        repo.upsert_account(account)
            .map_err(|error| error.to_string())?;
    }
    repo.set_settings(&request.settings)
        .map_err(|error| error.to_string())?;
    channel_state_get(repo).await
}

/// 连接渠道账号。

#[tauri::command]
pub async fn channel_connect(
    state: tauri::State<'_, crate::shared::state::AppState>,
    coordinator: State<'_, Arc<ChannelCoordinator>>,
    repo: State<'_, Arc<ChannelRepo>>,
    dispatcher: State<'_, Arc<ChannelDispatcher>>,
    account_id: String,
) -> DingDaResult<IpcResponse<ChannelIpcConnectResponse>> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(|error| error.to_string())?;

    let accounts = repo.list_accounts().map_err(|error| error.to_string())?;
    let account = accounts
        .iter()
        .find(|account| account.id == account_id)
        .cloned()
        .ok_or_else(|| format!("账号不存在: {account_id}"))?;

    dispatcher
        .connect(&account)
        .await
        .map_err(|error| error.to_string())?;

    let _ = coordinator; // 协调器由 dispatcher 的 listener 绑定触发。

    let state = dispatcher
        .connection_state(&account_id)
        .await
        .as_str()
        .to_string();
    Ok(IpcResponse::ok(ChannelIpcConnectResponse {
        ok: true,
        state,
        detail: None,
    }))
}

/// 断开渠道账号。

#[tauri::command]
pub async fn channel_disconnect(
    state: tauri::State<'_, crate::shared::state::AppState>,
    dispatcher: State<'_, Arc<ChannelDispatcher>>,
    account_id: String,
) -> DingDaResult<IpcResponse<ChannelIpcDisconnectResponse>> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(|error| error.to_string())?;
    dispatcher
        .disconnect(&account_id)
        .await
        .map_err(|error| error.to_string())?;
    Ok(IpcResponse::ok(ChannelIpcDisconnectResponse { ok: true }))
}

/// 人工发送消息。

#[tauri::command]
pub async fn channel_send(
    state: tauri::State<'_, crate::shared::state::AppState>,
    coordinator: State<'_, Arc<ChannelCoordinator>>,
    repo: State<'_, Arc<ChannelRepo>>,
    request: ChannelIpcSendRequest,
) -> DingDaResult<IpcResponse<ChannelIpcSendResponse>> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(|error| error.to_string())?;

    let conversation = repo
        .list_conversations()
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|conversation: &ChannelConversation| conversation.id == request.conversation_id)
        .ok_or_else(|| format!("会话不存在: {}", request.conversation_id))?;

    let message_id = coordinator
        .send_message(&conversation, &request.content)
        .await
        .map_err(|error| error.to_string())?;

    Ok(IpcResponse::ok(ChannelIpcSendResponse {
        ok: true,
        message_id,
        detail: None,
    }))
}
