//! 业务账号 → 渠道连接桥接 Tauri commands。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use crate::platforms::xianyu::ipc::account::AccountHandle;
use crate::shared::channel::dispatcher::ChannelDispatcher;
use crate::shared::ipc::IpcResponse;
use crate::shared::state::AppState;
use app::account::AccountStore;
use common::contracts::ChannelAccount;
use dingda_macros::timed;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 连接请求。
#[derive(Debug, Deserialize)]
pub struct AccountConnectRequest {
    pub owner_id: i64,
    pub account_id: String,
}

/// 由业务账号构造渠道账号（credential = cookie 字符串）。
pub fn to_channel_account(_owner_id: i64, account: &app::account::XianyuAccount) -> ChannelAccount {
    ChannelAccount {
        id: account.account_id.clone(),
        kind: "xianyu".to_string(),
        name: if account.display_name.is_empty() {
            account.account_id.clone()
        } else {
            account.display_name.clone()
        },
        credential: account.cookie.clone(),
        enabled: account.is_active(),
    }
}

/// 连接业务账号（建立渠道 websocket 设备监听）。
#[tauri::command]
#[timed]
pub async fn account_connect(
    state: State<'_, AppState>,
    accounts: State<'_, AccountHandle>,
    dispatcher: State<'_, Arc<ChannelDispatcher>>,
    request: AccountConnectRequest,
) -> common::DingDaResult<IpcResponse<String>> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(common::DingDaError::wrap)?;

    let account = accounts
        .store
        .get_account(request.owner_id, &request.account_id)
        .map_err(common::DingDaError::wrap)?
        .ok_or_else(|| format!("账号不存在: {}", request.account_id))?;
    if !account.has_cookie() {
        return Err("账号缺少 Cookie，请先扫码登录".into());
    }

    let channel_account = to_channel_account(request.owner_id, &account);
    tracing::info!(account = %request.account_id, "开始连接闲鱼并绑定设备监听");
    dispatcher
        .connect(&channel_account)
        .await
        .map_err(common::DingDaError::wrap)?;
    Ok(IpcResponse::ok(
        dispatcher
            .connection_state(&request.account_id)
            .await
            .as_str()
            .to_string(),
    ))
}

/// 断开业务账号的渠道连接。
#[tauri::command]
#[timed]
pub async fn account_disconnect(
    state: State<'_, AppState>,
    dispatcher: State<'_, Arc<ChannelDispatcher>>,
    request: AccountConnectRequest,
) -> common::DingDaResult<IpcResponse<()>> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(common::DingDaError::wrap)?;

    tracing::info!(account = %request.account_id, "断开闲鱼连接");
    dispatcher
        .disconnect(&request.account_id)
        .await
        .map_err(common::DingDaError::wrap)?;
    Ok(IpcResponse::ok(()))
}

/// 查询业务账号的渠道连接状态。
#[tauri::command]
pub async fn account_connection_state(
    state: State<'_, AppState>,
    dispatcher: State<'_, Arc<ChannelDispatcher>>,
    request: AccountConnectRequest,
) -> common::DingDaResult<IpcResponse<String>> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(common::DingDaError::wrap)?;

    Ok(IpcResponse::ok(
        dispatcher
            .connection_state(&request.account_id)
            .await
            .as_str()
            .to_string(),
    ))
}
