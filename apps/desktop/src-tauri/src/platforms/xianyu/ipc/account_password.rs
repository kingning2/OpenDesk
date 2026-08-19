//! 业务账号密码登录 Tauri commands — 通过 sidecar Playwright 登录并导出 cookies。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use crate::platforms::xianyu::ipc::account::AccountHandle;
use crate::shared::channel::dispatcher::ChannelDispatcher;
use crate::shared::ipc::IpcResponse;
use crate::shared::state::AppState;
use app::account::{AccountService, AccountStore, AccountUpdate, LoginMethod, XianyuAccount};
use common::contracts::ChannelCookie;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

/// 账号密码登录入参。
#[derive(Debug, Deserialize)]
pub struct AccountPasswordLoginRequest {
    pub login_id: String,
    pub password: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// 账号密码登录响应。
#[derive(Debug, Serialize)]
pub struct AccountPasswordLoginResponse {
    pub ok: bool,
    pub status: String,
    pub account_id: Option<String>,
    pub detail: Option<String>,
}

/// sidecar 账号密码登录请求。
#[derive(Debug, Serialize)]
struct SidecarPasswordLoginRequest {
    login_id: String,
    password: String,
    trace_id: Option<String>,
}

/// sidecar 账号密码登录响应。
#[derive(Debug, Deserialize)]
struct SidecarPasswordLoginResponse {
    ok: bool,
    status: String,
    cookies: Option<Vec<ChannelCookie>>,
    detail: Option<String>,
}

/// 使用账号密码登录业务账号。
#[tauri::command]
pub async fn account_password_login(
    state: State<'_, AppState>,
    app: AppHandle,
    accounts: State<'_, AccountHandle>,
    dispatcher: State<'_, std::sync::Arc<ChannelDispatcher>>,
    request: AccountPasswordLoginRequest,
) -> common::DingDaResult<IpcResponse<AccountPasswordLoginResponse>> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(common::DingDaError::wrap)?;

    let sidecar_request = SidecarPasswordLoginRequest {
        login_id: request.login_id.clone(),
        password: request.password.clone(),
        trace_id: Some(
            request
                .name
                .clone()
                .unwrap_or_else(|| "account-password".to_string()),
        ),
    };
    let sidecar = state.lifecycle.client();
    let response: SidecarPasswordLoginResponse = sidecar
        .post_json("/v1/channel/password_login", &sidecar_request)
        .await
        .map_err(common::DingDaError::wrap)?;

    if response.status != "success" || !response.ok {
        return Ok(IpcResponse::ok(AccountPasswordLoginResponse {
            ok: response.ok,
            status: response.status,
            account_id: None,
            detail: response.detail,
        }));
    }

    let Some(cookies) = response.cookies else {
        return Ok(IpcResponse::ok(AccountPasswordLoginResponse {
            ok: false,
            status: "failed".to_string(),
            account_id: None,
            detail: Some("登录成功但未导出 cookies".to_string()),
        }));
    };

    let mut account = account_from_cookies(&cookies);
    account.login_id = request.login_id.clone();
    account.login_password = request.password.clone();
    if let Some(name) = request.name {
        if !name.trim().is_empty() {
            account.display_name = name.trim().to_string();
        }
    }
    account.login_method = LoginMethod::Password;
    account.remark = "账号密码登录".to_string();

    let _ = app;
    let service = AccountService::new(accounts.store.as_ref());

    match accounts.store.get_account(1, &account.account_id) {
        Ok(Some(_)) => {
            service
                .update(
                    1,
                    &account.account_id,
                    &AccountUpdate {
                        cookie: Some(account.cookie.clone()),
                        unb: Some(account.unb.clone()),
                        login_id: Some(account.login_id.clone()),
                        login_password: Some(account.login_password.clone()),
                        login_method: Some(LoginMethod::Password),
                        display_name: if account.display_name.is_empty() {
                            None
                        } else {
                            Some(account.display_name.clone())
                        },
                        last_login_at: Some(now_string()),
                        ..Default::default()
                    },
                )
                .map_err(common::DingDaError::wrap)?;
        }
        _ => {
            service
                .create(1, &account)
                .map_err(common::DingDaError::wrap)?;
        }
    }

    let channel_account = super::account_connection::to_channel_account(1, &account);
    info!(account = %account.account_id, "账号密码登录成功，自动连接闲鱼");
    dispatcher
        .connect(&channel_account)
        .await
        .map_err(common::DingDaError::wrap)?;

    Ok(IpcResponse::ok(AccountPasswordLoginResponse {
        ok: true,
        status: "success".to_string(),
        account_id: Some(account.account_id),
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
            "xianyu-password".to_string()
        } else {
            unb.clone()
        },
        display_name: String::new(),
        avatar_url: String::new(),
        login_id: String::new(),
        login_password: String::new(),
        unb,
        cookie: cookie_str,
        login_method: LoginMethod::Password,
        status: app::account::AccountStatus::Active,
        remark: "账号密码登录".to_string(),
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
