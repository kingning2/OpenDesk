//! 主窗口内嵌子 WebView 显示闲鱼页面 — 注入 cookies 保持登录态。
//!
//! 负责：
//! - 在主窗口指定 bounds 创建/更新子 WebView（Tauri `unstable` + `add_child`）
//! - 注入账号 credential 中的 goofish cookies
//! - 关闭时销毁子 WebView
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-13

use common::contracts::{ChannelAccount, ChannelCookie};
use common::OpenDeskResult;
use tauri::webview::cookie::Cookie;
use tauri::webview::WebviewBuilder;
use tauri::{LogicalPosition, LogicalSize, Manager, Webview, WebviewUrl};

/// 内嵌闲鱼子 WebView 标签（挂在主窗口下，非独立窗）。
pub const XIANYU_WEBVIEW_LABEL: &str = "xianyu-site";

const MAIN_WINDOW_LABEL: &str = "main";
const GOOFISH_URL: &str = common::constants::xianyu::WEB_ORIGIN;

/// 打开或调整主窗口内嵌的闲鱼子 WebView。
///
/// 已存在时只更新位置/尺寸；首次创建时注入 cookies 并导航到闲鱼首页。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-13
///
/// # 参数
/// - `app` — Tauri 应用句柄
/// - `account` — 含 credential（cookies）的渠道账号
/// - `x` / `y` / `width` / `height` — 相对主窗口客户区的逻辑像素 bounds
///
/// # 返回值
/// 成功返回 `Ok(())`；缺少主窗口、无 cookies 或创建失败时返回错误文案。
pub fn open_xianyu_site(
    app: &tauri::AppHandle,
    account: &ChannelAccount,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> OpenDeskResult<()> {
    if width < 8.0 || height < 8.0 {
        return Err("内嵌区域过小".into());
    }

    let position = LogicalPosition::new(x, y);
    let size = LogicalSize::new(width, height);

    // 已存在：只同步布局（前端 ResizeObserver / StrictMode 重挂载会反复调用）。
    if let Some(webview) = app.get_webview(XIANYU_WEBVIEW_LABEL) {
        return apply_bounds(&webview, position, size);
    }

    // 清理旧版独立窗口（若仍残留）。
    if let Some(legacy) = app.get_webview_window(XIANYU_WEBVIEW_LABEL) {
        let _ = legacy.destroy();
    }

    let cookies = platform::xianyu::cookies::parse_credential(&account.credential);
    if cookies.is_empty() {
        return Err("账号凭据中没有 cookies，请先扫码登录".into());
    }

    let blank = "about:blank"
        .parse::<url::Url>()
        .map_err(|error| format!("about:blank 解析失败: {error}"))?;
    let target = GOOFISH_URL
        .parse::<url::Url>()
        .map_err(|error| format!("闲鱼 URL 解析失败: {error}"))?;

    let window = app
        .get_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| "主窗口不存在".to_string())?;

    let builder = WebviewBuilder::new(XIANYU_WEBVIEW_LABEL, WebviewUrl::External(blank))
        .on_navigation(|url| {
            let href = url.as_str();
            href == "about:blank" || href.starts_with(common::constants::xianyu::WEB_ORIGIN)
        });

    let webview = match window.add_child(builder, position, size) {
        Ok(webview) => webview,
        Err(error) => {
            let message = error.to_string();
            // close 与重建竞态：标签仍在但 get_webview 曾短暂为空。
            if message.contains("already exists") {
                if let Some(existing) = app.get_webview(XIANYU_WEBVIEW_LABEL) {
                    tracing::info!("闲鱼内嵌视图已存在，改为同步布局");
                    return apply_bounds(&existing, position, size);
                }
            }
            return Err(format!("创建闲鱼内嵌视图失败: {message}").into());
        }
    };

    let mut injected = 0usize;
    for cookie in cookies
        .iter()
        .filter(|cookie| cookie_matches_goofish(cookie))
    {
        match inject_cookie(&webview, cookie) {
            Ok(()) => injected += 1,
            Err(error) => {
                tracing::warn!(%error, cookie = %cookie.name, "注入 cookie 失败");
            }
        }
    }
    tracing::info!(
        account = %account.id,
        injected,
        total = cookies.len(),
        "闲鱼内嵌视图已创建并注入 cookies"
    );

    webview
        .navigate(target)
        .map_err(|error| format!("导航闲鱼页面失败: {error}"))?;

    Ok(())
}

/// 关闭并销毁主窗口内嵌的闲鱼子 WebView。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-13
///
/// # 参数
/// - `app` — Tauri 应用句柄
///
/// # 返回值
/// 成功或不存在时返回 `Ok(())`。
pub fn close_xianyu_site(app: &tauri::AppHandle) -> OpenDeskResult<()> {
    if let Some(webview) = app.get_webview(XIANYU_WEBVIEW_LABEL) {
        webview
            .close()
            .map_err(|error| format!("关闭闲鱼内嵌视图失败: {error}"))?;
        tracing::info!("已关闭闲鱼内嵌视图");
    }
    // 兼容：若旧独立窗仍在，一并销毁。
    if let Some(legacy) = app.get_webview_window(XIANYU_WEBVIEW_LABEL) {
        let _ = legacy.destroy();
    }
    Ok(())
}

fn apply_bounds(
    webview: &Webview,
    position: LogicalPosition<f64>,
    size: LogicalSize<f64>,
) -> OpenDeskResult<()> {
    webview
        .set_position(position)
        .map_err(|error| format!("更新闲鱼位置失败: {error}"))?;
    webview
        .set_size(size)
        .map_err(|error| format!("更新闲鱼尺寸失败: {error}"))?;
    Ok(())
}

fn cookie_matches_goofish(cookie: &ChannelCookie) -> bool {
    let domain = cookie.domain.to_ascii_lowercase();
    domain.is_empty() || domain.contains(common::constants::xianyu::COOKIE_DOMAIN)
}

fn inject_cookie(webview: &Webview, cookie: &ChannelCookie) -> OpenDeskResult<()> {
    let base = Cookie::new(cookie.name.clone(), cookie.value.clone());
    let mut builder = Cookie::build(base);

    if !cookie.domain.is_empty() {
        builder = builder.domain(cookie.domain.clone());
    }
    if !cookie.path.is_empty() {
        builder = builder.path(cookie.path.clone());
    }
    if let Some(secure) = cookie.secure {
        builder = builder.secure(secure);
    }
    if let Some(http_only) = cookie.http_only {
        builder = builder.http_only(http_only);
    }
    match cookie.same_site.as_deref() {
        Some("None") => builder = builder.same_site(tauri::webview::cookie::SameSite::None),
        Some("Strict") => builder = builder.same_site(tauri::webview::cookie::SameSite::Strict),
        _ => builder = builder.same_site(tauri::webview::cookie::SameSite::Lax),
    }

    let cookie = builder.build();
    Ok(webview
        .set_cookie(cookie)
        .map_err(|error| format!("set_cookie 失败: {error}"))?)
}
