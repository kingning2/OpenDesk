//! 闲鱼内嵌站点 Tauri commands — 打开/关闭主窗口子 WebView。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use crate::platforms::xianyu::webview;
use crate::shared::channel::ChannelRepo;
use crate::shared::state::AppState;
use common::contracts::{
    ChannelIpcCloseSiteResponse, ChannelIpcOpenSiteRequest, ChannelIpcOpenSiteResponse,
};
use std::sync::Arc;
use tauri::{AppHandle, State};

/// 打开主窗口内嵌闲鱼页面（子 WebView + 注入 cookies）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
///
/// # 参数
///
/// * `state` — 应用状态（License 校验）
/// * `repo` — 渠道账号仓库
/// * `app` — Tauri 应用句柄
/// * `request` — 账号 id 与相对主窗口客户区的逻辑像素 bounds
///
/// # 返回值
///
/// 成功返回 `ok: true`；账号不存在、无 cookies 或创建 WebView 失败时返回错误文案。
#[tauri::command]
pub async fn channel_open_site(
    state: State<'_, AppState>,
    repo: State<'_, Arc<ChannelRepo>>,
    app: AppHandle,
    request: ChannelIpcOpenSiteRequest,
) -> common::OpenDeskResult<ChannelIpcOpenSiteResponse> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(common::OpenDeskError::wrap)?;

    let account = repo
        .list_accounts()
        .map_err(common::OpenDeskError::wrap)?
        .into_iter()
        .find(|account| account.id == request.account_id)
        .ok_or_else(|| format!("账号不存在: {}", request.account_id))?;

    webview::open_xianyu_site(
        &app,
        &account,
        request.x,
        request.y,
        request.width,
        request.height,
    )?;
    Ok(ChannelIpcOpenSiteResponse {
        ok: true,
        detail: None,
    })
}

/// 关闭主窗口内嵌的闲鱼子 WebView。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
///
/// # 参数
///
/// * `state` — 应用状态（License 校验）
/// * `app` — Tauri 应用句柄
///
/// # 返回值
///
/// 成功或不存在时返回 `ok: true`。
#[tauri::command]
pub async fn channel_close_site(
    state: State<'_, AppState>,
    app: AppHandle,
) -> common::OpenDeskResult<ChannelIpcCloseSiteResponse> {
    state
        .license
        .ensure_licensed()
        .await
        .map_err(common::OpenDeskError::wrap)?;
    webview::close_xianyu_site(&app)?;
    Ok(ChannelIpcCloseSiteResponse { ok: true })
}
