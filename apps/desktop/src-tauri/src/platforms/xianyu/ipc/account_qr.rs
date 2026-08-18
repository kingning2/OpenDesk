//! 业务账号扫码登录 Tauri commands — 扫码创建/绑定业务账号。
//!
//! 复用 sidecar 的 `channel_qr_*` 扫码能力（Playwright 二维码 + 轮询），
//! 登录成功后把 cookies 写入 crates/app 账号层（`InMemoryAccountStore`），
//! 与渠道账号扫码（写 ChannelRepo）职责分离。

use crate::platforms::xianyu::persist::InMemoryAccountStore;
use crate::shared::channel::dispatcher::ChannelDispatcher;
use crate::shared::ipc::IpcResponse;
use crate::shared::state::AppState;
use app::account::{AccountService, AccountStore, AccountUpdate, LoginMethod, XianyuAccount};
use common::contracts::{
    ChannelCookie, ChannelIpcQrCancelRequest, ChannelIpcQrCancelResponse, ChannelIpcQrCheckRequest,
    ChannelIpcQrCheckResponse, ChannelIpcQrStartResponse, ChannelSidecarQrCancelRequest,
    ChannelSidecarQrCheckRequest, ChannelSidecarQrStartRequest,
};
use opendesk_macros::timed;
use serde::Deserialize;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

/// 业务账号扫码服务句柄（setup 时注册到 Tauri 状态）。
pub struct AccountQrHandle {
    pub store: Arc<InMemoryAccountStore>,
}

/// 扫码成功后创建账号的入参（前端可选覆盖）。
#[derive(Debug, Deserialize)]
pub struct AccountQrStartRequest {
    /// 展示名称（可选；默认「闲鱼账号」）。
    #[serde(default)]
    pub name: Option<String>,
}

/// 启动业务账号扫码登录。
#[tauri::command]
#[timed]
pub async fn account_qr_start(
    state: State<'_, AppState>,
    request: AccountQrStartRequest,
) -> common::OpenDeskResult<IpcResponse<ChannelIpcQrStartResponse>> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(common::OpenDeskError::wrap)?;

    let sidecar_request = ChannelSidecarQrStartRequest {
        account_id: String::new(),
        trace_id: Some(
            request
                .name
                .clone()
                .unwrap_or_else(|| "account-qr".to_string()),
        ),
    };
    let sidecar = state.lifecycle.client();
    let response = runtime::sidecar::routes::channel_qr_start::call(sidecar, sidecar_request)
        .await
        .map_err(common::OpenDeskError::wrap)?;

    Ok(IpcResponse::ok(ChannelIpcQrStartResponse {
        ok: response.ok,
        status: response.status,
        session_id: response.session_id,
        qr_base64: response.qr_base64,
        detail: response.detail,
    }))
}

/// 轮询扫码状态；登录成功后自动创建业务账号。
#[tauri::command]
#[timed]
pub async fn account_qr_check(
    state: State<'_, AppState>,
    app: AppHandle,
    dispatcher: State<'_, Arc<ChannelDispatcher>>,
    request: ChannelIpcQrCheckRequest,
) -> common::OpenDeskResult<IpcResponse<ChannelIpcQrCheckResponse>> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(common::OpenDeskError::wrap)?;

    let sidecar_request = ChannelSidecarQrCheckRequest {
        session_id: request.session_id.clone(),
        trace_id: Some(request.session_id.clone()),
    };
    let sidecar = state.lifecycle.client();
    let response = runtime::sidecar::routes::channel_qr_check::call(sidecar, sidecar_request)
        .await
        .map_err(common::OpenDeskError::wrap)?;

    // 登录成功：写入业务账号层（自动创建账号），并自动建立渠道连接
    if response.status == "success" {
        if let Some(cookies) = response.cookies.clone() {
            let account = account_from_cookies(&cookies);
            let handle = app.state::<AccountQrHandle>();
            let service = AccountService::new(handle.store.as_ref());

            match handle.store.get_account(1, &account.account_id) {
                Ok(Some(_)) => {
                    service
                        .update(
                            1,
                            &account.account_id,
                            &AccountUpdate {
                                cookie: Some(account.cookie.clone()),
                                unb: Some(account.unb.clone()),
                                login_method: Some(LoginMethod::Qr),
                                last_login_at: Some(now_string()),
                                ..Default::default()
                            },
                        )
                        .map_err(common::OpenDeskError::wrap)?;
                }
                _ => {
                    service
                        .create(1, &account)
                        .map_err(common::OpenDeskError::wrap)?;
                }
            }

            // 扫码成功后即时连接（用户手动连接仍保留）
            let channel_account = super::account_connection::to_channel_account(1, &account);
            tracing::info!(account = %account.account_id, "扫码成功，自动连接闲鱼");
            dispatcher
                .connect(&channel_account)
                .await
                .map_err(common::OpenDeskError::wrap)?;
        }
    }

    Ok(IpcResponse::ok(ChannelIpcQrCheckResponse {
        ok: response.ok,
        status: response.status,
        session_id: response.session_id,
        cookies: response.cookies,
        detail: response.detail,
        qr_base64: response.qr_base64,
    }))
}

/// 取消业务账号扫码登录。
#[tauri::command]
#[timed]
pub async fn account_qr_cancel(
    state: State<'_, AppState>,
    request: ChannelIpcQrCancelRequest,
) -> common::OpenDeskResult<IpcResponse<ChannelIpcQrCancelResponse>> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(common::OpenDeskError::wrap)?;

    let sidecar_request = ChannelSidecarQrCancelRequest {
        session_id: request.session_id.clone(),
        trace_id: Some(request.session_id.clone()),
    };
    let sidecar = state.lifecycle.client();
    let response = runtime::sidecar::routes::channel_qr_cancel::call(sidecar, sidecar_request)
        .await
        .map_err(common::OpenDeskError::wrap)?;

    Ok(IpcResponse::ok(ChannelIpcQrCancelResponse {
        ok: response.ok,
        detail: response.detail,
    }))
}

/// 从 cookies 构造业务账号：unb 作为 account_id，cookie 序列化为字符串。
fn account_from_cookies(cookies: &[ChannelCookie]) -> XianyuAccount {
    let unb = cookies
        .iter()
        .find(|cookie| cookie.name == "unb")
        .map(|cookie| cookie.value.clone())
        .unwrap_or_default();
    let cookie_str = cookies
        .iter()
        .map(|cookie| format!("{}={}", cookie.name, cookie.value))
        .collect::<Vec<_>>()
        .join("; ");

    XianyuAccount {
        id: 0,
        owner_id: 1,
        account_id: if unb.is_empty() {
            "xianyu-qr".to_string()
        } else {
            unb.clone()
        },
        display_name: String::new(),
        login_id: String::new(),
        login_password: String::new(),
        unb,
        cookie: cookie_str,
        login_method: LoginMethod::Qr,
        status: app::account::AccountStatus::Active,
        remark: "扫码登录".to_string(),
        pause_duration_minutes: 10,
        last_login_at: Some(now_string()),
        last_refresh_at: None,
        proxy: app::account::ProxyConfig::default(),
        automation: app::account::AccountAutomation::default(),
        delivery_guard: app::account::DeliveryGuard::default(),
    }
}

fn now_string() -> String {
    use chrono::Utc;
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
