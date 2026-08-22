//! 业务账号 → 渠道连接桥接 Tauri commands。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use crate::platforms::xianyu::ipc::account::AccountHandle;
use crate::platforms::xianyu::persist::InMemoryAccountStore;
use crate::shared::channel::dispatcher::ChannelDispatcher;
use crate::shared::ipc::IpcResponse;
use crate::shared::state::AppState;
use app::account::{AccountService, AccountStore, AccountUpdate};
use common::contracts::ChannelAccount;
use platform::xianyu::fetch_user_profile;
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

/// 连接成功后拉取闲鱼用户资料并写回业务账号（昵称 / 头像 / Cookie）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
///
/// # 参数
///
/// * `store` — 业务账号存储
/// * `owner_id` — 归属用户 id
/// * `account_id` — 账号标识
///
/// # 返回值
///
/// 成功返回 `()`；拉取或写入失败返回错误文案。
pub async fn sync_account_profile(
    store: &InMemoryAccountStore,
    owner_id: i64,
    account_id: &str,
) -> common::DingDaResult<()> {
    let account = store
        .get_account(owner_id, account_id)
        .map_err(common::DingDaError::wrap)?
        .ok_or_else(|| format!("账号不存在: {account_id}"))?;
    if !account.has_cookie() {
        return Err("账号缺少 Cookie".into());
    }

    let (profile, cookie) = fetch_user_profile(&account.cookie)
        .await
        .map_err(common::DingDaError::wrap)?;

    let service = AccountService::new(store);
    let patch = AccountUpdate {
        display_name: if profile.display_name.is_empty() {
            None
        } else {
            Some(profile.display_name.clone())
        },
        avatar_url: if profile.avatar_url.is_empty() {
            None
        } else {
            Some(profile.avatar_url.clone())
        },
        cookie: if cookie != account.cookie {
            Some(cookie)
        } else {
            None
        },
        ..Default::default()
    };

    if patch.display_name.is_none() && patch.avatar_url.is_none() && patch.cookie.is_none() {
        return Ok(());
    }

    service
        .update(owner_id, account_id, &patch)
        .map_err(common::DingDaError::wrap)?;

    info!(
        account = %account_id,
        display_name = %profile.display_name,
        has_avatar = !profile.avatar_url.is_empty(),
        "闲鱼用户资料已同步"
    );
    Ok(())
}

/// 连接业务账号（建立渠道 websocket 设备监听）。
#[tauri::command]
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
    info!(account = %request.account_id, "开始连接闲鱼并绑定设备监听");
    dispatcher
        .connect(&channel_account)
        .await
        .map_err(common::DingDaError::wrap)?;

    if let Err(error) =
        sync_account_profile(&accounts.store, request.owner_id, &request.account_id).await
    {
        let text = error.to_string();
        // Session 过期不是「连接仍可用」：断开并让前端提示重新登录。
        if text.contains("FAIL_SYS_SESSION_EXPIRED")
            || text.contains("Session过期")
            || text.contains("SESSION_EXPIRED")
        {
            warn!(
                account = %request.account_id,
                %error,
                "登录态已过期，断开连接并提示重新登录"
            );
            let _ = dispatcher.disconnect(&request.account_id).await;
            return Err("登录态已过期，请重新扫码登录".into());
        }
        warn!(
            account = %request.account_id,
            %error,
            "拉取闲鱼用户资料失败，连接仍可用"
        );
    }

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

    info!(account = %request.account_id, "断开闲鱼连接");
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

/// 手动触发浏览器滑块续期（打开 Playwright 窗口完成验证）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-20
#[tauri::command]
pub async fn account_cookie_renew(
    state: State<'_, AppState>,
    renewer: State<'_, Arc<crate::shared::channel::cookie_renew::RiskCookieRenewer>>,
    request: AccountConnectRequest,
) -> common::DingDaResult<IpcResponse<String>> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(common::DingDaError::wrap)?;

    info!(account = %request.account_id, "手动触发闲鱼滑块续期");
    let renewer = Arc::clone(&renewer);
    let schedule = &renewer.spawn_renew(request.account_id.clone(), String::new());
    let message = match schedule {
        crate::shared::channel::cookie_renew::RenewSchedule::Started => "已开始滑块续期",
        crate::shared::channel::cookie_renew::RenewSchedule::Queued => "已加入续期队列，请稍候",
        crate::shared::channel::cookie_renew::RenewSchedule::InFlight => "续期已在进行或排队中",
        crate::shared::channel::cookie_renew::RenewSchedule::Cooldown => "续期冷却中，请稍后再试",
        crate::shared::channel::cookie_renew::RenewSchedule::Disabled => "自动续期未启用",
    };
    Ok(IpcResponse::ok(message.to_string()))
}
