//! 渠道相关 Tauri commands。

use common::contracts::{
    ChannelConversation, ChannelIpcCloseSiteResponse, ChannelIpcConnectResponse,
    ChannelIpcDisconnectResponse, ChannelIpcLoginRequest, ChannelIpcLoginResponse,
    ChannelIpcOpenSiteRequest, ChannelIpcOpenSiteResponse, ChannelIpcQrCancelRequest,
    ChannelIpcQrCancelResponse, ChannelIpcQrCheckRequest, ChannelIpcQrCheckResponse,
    ChannelIpcQrStartRequest, ChannelIpcQrStartResponse, ChannelIpcSendRequest,
    ChannelIpcSendResponse, ChannelIpcStateRequest, ChannelIpcStateResponse,
    ChannelSidecarLoginRequest, ChannelSidecarQrCancelRequest, ChannelSidecarQrCheckRequest,
    ChannelSidecarQrStartRequest,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, State};

use super::coordinator::ChannelCoordinator;
use super::dispatcher::ChannelDispatcher;
use super::store::ChannelRepo;
use super::webview;

/// QR 扫码会话 → 账号 id 映射（qr_start 登记，qr_check 消费）。
fn qr_account_map() -> &'static Mutex<HashMap<String, String>> {
    static MAP: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 读取渠道全量状态（账号/会话/消息/设置）。
#[tauri::command]
pub async fn channel_state_get(
    repo: State<'_, Arc<ChannelRepo>>,
) -> Result<ChannelIpcStateResponse, String> {
    let accounts = repo.list_accounts().map_err(|error| error.to_string())?;
    let conversations = repo
        .list_conversations()
        .map_err(|error| error.to_string())?;
    let messages = repo.list_all_messages().map_err(|error| error.to_string())?;
    let settings = repo.get_settings().map_err(|error| error.to_string())?;
    Ok(ChannelIpcStateResponse {
        accounts,
        conversations,
        messages,
        settings,
    })
}

/// 保存渠道配置（账号 + 设置）。
#[tauri::command]
pub async fn channel_state_set(
    repo: State<'_, Arc<ChannelRepo>>,
    request: ChannelIpcStateRequest,
) -> Result<ChannelIpcStateResponse, String> {
    for account in &request.accounts {
        repo.upsert_account(account).map_err(|error| error.to_string())?;
    }
    repo.set_settings(&request.settings)
        .map_err(|error| error.to_string())?;
    channel_state_get(repo).await
}

/// 连接渠道账号。
#[tauri::command]
pub async fn channel_connect(
    state: tauri::State<'_, crate::state::AppState>,
    coordinator: State<'_, Arc<ChannelCoordinator>>,
    repo: State<'_, Arc<ChannelRepo>>,
    dispatcher: State<'_, Arc<ChannelDispatcher>>,
    account_id: String,
) -> Result<ChannelIpcConnectResponse, String> {
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
    Ok(ChannelIpcConnectResponse {
        ok: true,
        state,
        detail: None,
    })
}

/// 断开渠道账号。
#[tauri::command]
pub async fn channel_disconnect(
    state: tauri::State<'_, crate::state::AppState>,
    dispatcher: State<'_, Arc<ChannelDispatcher>>,
    account_id: String,
) -> Result<ChannelIpcDisconnectResponse, String> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(|error| error.to_string())?;
    dispatcher
        .disconnect(&account_id)
        .await
        .map_err(|error| error.to_string())?;
    Ok(ChannelIpcDisconnectResponse { ok: true })
}

/// 人工发送消息。
#[tauri::command]
pub async fn channel_send(
    state: tauri::State<'_, crate::state::AppState>,
    coordinator: State<'_, Arc<ChannelCoordinator>>,
    repo: State<'_, Arc<ChannelRepo>>,
    request: ChannelIpcSendRequest,
) -> Result<ChannelIpcSendResponse, String> {
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
        .await?;

    Ok(ChannelIpcSendResponse {
        ok: true,
        message_id,
        detail: None,
    })
}

/// 用浏览器快照登录：调 Python Playwright 恢复会话并导出 cookies，更新账号凭据。
#[tauri::command]
pub async fn channel_login(
    state: tauri::State<'_, crate::state::AppState>,
    repo: State<'_, Arc<ChannelRepo>>,
    dispatcher: State<'_, Arc<ChannelDispatcher>>,
    request: ChannelIpcLoginRequest,
) -> Result<ChannelIpcLoginResponse, String> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(|error| error.to_string())?;

    let account = repo
        .list_accounts()
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|account| account.id == request.account_id)
        .ok_or_else(|| format!("账号不存在: {}", request.account_id))?;

    let sidecar_request = ChannelSidecarLoginRequest {
        account_id: account.id.clone(),
        credential: account.credential.clone(),
        trace_id: Some(request.account_id.clone()),
    };
    let sidecar = state.lifecycle.client();
    let response = runtime::sidecar::routes::channel_login::call(sidecar, sidecar_request)
        .await
        .map_err(|error| error.to_string())?;

    if !response.ok {
        return Ok(ChannelIpcLoginResponse {
            ok: false,
            state: "error".to_string(),
            cookies: None,
            detail: response.detail.or(Some("登录失败".to_string())),
        });
    }

    let cookies = response.cookies.unwrap_or_default();

    // 登录成功：更新凭据为导出的 cookies 数组 JSON，然后连接。
    if !cookies.is_empty() {
        let credential = serde_json::to_string(&cookies).unwrap_or_default();
        let mut updated = account.clone();
        updated.credential = credential;
        repo.upsert_account(&updated).map_err(|error| error.to_string())?;
        dispatcher.connect(&updated).await.map_err(|error| error.to_string())?;
    }

    Ok(ChannelIpcLoginResponse {
        ok: true,
        state: "connected".to_string(),
        cookies: Some(cookies),
        detail: response.detail,
    })
}

/// 打开内嵌闲鱼页面（Webview + 注入 cookies）。
#[tauri::command]
pub async fn channel_open_site(
    state: tauri::State<'_, crate::state::AppState>,
    repo: State<'_, Arc<ChannelRepo>>,
    app: AppHandle,
    request: ChannelIpcOpenSiteRequest,
) -> Result<ChannelIpcOpenSiteResponse, String> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(|error| error.to_string())?;

    let account = repo
        .list_accounts()
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|account| account.id == request.account_id)
        .ok_or_else(|| format!("账号不存在: {}", request.account_id))?;

    webview::open_xianyu_site(&app, &account)?;
    Ok(ChannelIpcOpenSiteResponse { ok: true, detail: None })
}

/// 关闭内嵌闲鱼页面。
#[tauri::command]
pub async fn channel_close_site(
    state: tauri::State<'_, crate::state::AppState>,
    app: AppHandle,
) -> Result<ChannelIpcCloseSiteResponse, String> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(|error| error.to_string())?;
    webview::close_xianyu_site(&app)?;
    Ok(ChannelIpcCloseSiteResponse { ok: true })
}

/// 启动扫码登录：调 Python Playwright 打开登录页，截图二维码。
#[tauri::command]
pub async fn channel_qr_start(
    state: tauri::State<'_, crate::state::AppState>,
    repo: State<'_, Arc<ChannelRepo>>,
    request: ChannelIpcQrStartRequest,
) -> Result<ChannelIpcQrStartResponse, String> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(|error| error.to_string())?;

    let account = repo
        .list_accounts()
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|account| account.id == request.account_id)
        .ok_or_else(|| format!("账号不存在: {}", request.account_id))?;

    let sidecar_request = ChannelSidecarQrStartRequest {
        account_id: account.id.clone(),
        trace_id: Some(request.account_id.clone()),
    };
    let sidecar = state.lifecycle.client();
    let response = runtime::sidecar::routes::channel_qr_start::call(sidecar, sidecar_request)
        .await
        .map_err(|error| error.to_string())?;

    // 登记 session → account 映射（qr_check 时消费）。
    if let Some(session_id) = response.session_id.clone() {
        let mut map = qr_account_map().lock().expect("qr account map lock");
        map.insert(session_id, account.id.clone());
    }

    Ok(ChannelIpcQrStartResponse {
        ok: response.ok,
        status: response.status,
        session_id: response.session_id,
        qr_base64: response.qr_base64,
        detail: response.detail,
    })
}

/// 轮询扫码状态；登录成功时更新账号凭据并连接。
#[tauri::command]
pub async fn channel_qr_check(
    state: tauri::State<'_, crate::state::AppState>,
    repo: State<'_, Arc<ChannelRepo>>,
    dispatcher: State<'_, Arc<ChannelDispatcher>>,
    request: ChannelIpcQrCheckRequest,
) -> Result<ChannelIpcQrCheckResponse, String> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(|error| error.to_string())?;

    let sidecar_request = ChannelSidecarQrCheckRequest {
        session_id: request.session_id.clone(),
        trace_id: Some(request.session_id.clone()),
    };
    let sidecar = state.lifecycle.client();
    let response = runtime::sidecar::routes::channel_qr_check::call(sidecar, sidecar_request)
        .await
        .map_err(|error| error.to_string())?;

    // 登录成功：更新账号凭据（存 cookies）+ 连接。
    if response.status == "success" {
        if let Some(cookies) = response.cookies.clone() {
            let credential = serde_json::to_string(&cookies).unwrap_or_default();
            let account_id = {
                let mut map = qr_account_map().lock().expect("qr account map lock");
                map.remove(&request.session_id)
            };
            let accounts = repo.list_accounts().map_err(|error| error.to_string())?;
            let target = account_id
                .and_then(|id| accounts.iter().find(|account| account.id == id))
                .cloned()
                .or_else(|| accounts.into_iter().find(|account| account.kind == "xianyu"));
            if let Some(account) = target {
                let mut updated = account;
                updated.credential = credential;
                repo.upsert_account(&updated).map_err(|error| error.to_string())?;
                let _ = dispatcher.connect(&updated).await;
            }
        }
    }

    Ok(ChannelIpcQrCheckResponse {
        ok: response.ok,
        status: response.status,
        session_id: response.session_id,
        cookies: response.cookies,
        detail: response.detail,
    })
}

/// 取消扫码登录。
#[tauri::command]
pub async fn channel_qr_cancel(
    state: tauri::State<'_, crate::state::AppState>,
    request: ChannelIpcQrCancelRequest,
) -> Result<ChannelIpcQrCancelResponse, String> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(|error| error.to_string())?;

    let sidecar_request = ChannelSidecarQrCancelRequest {
        session_id: request.session_id.clone(),
        trace_id: Some(request.session_id.clone()),
    };
    let sidecar = state.lifecycle.client();
    let response = runtime::sidecar::routes::channel_qr_cancel::call(sidecar, sidecar_request)
        .await
        .map_err(|error| error.to_string())?;

    Ok(ChannelIpcQrCancelResponse {
        ok: response.ok,
        detail: response.detail,
    })
}
