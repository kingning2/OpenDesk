//! 内嵌 Webview 显示闲鱼页面 — 注入快照 cookies 保持登录态。
//!
//! 登录载体为浏览器快照：Playwright 登录后导出的 cookies 注入 WebView2，
//! 使桌面内嵌窗口可显示已登录的 goofish.com 页面。

use common::contracts::{ChannelAccount, ChannelCookie};
use tauri::webview::cookie::Cookie;
use tauri::{Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// 内嵌闲鱼窗口标签。
pub const XIANYU_WEBVIEW_LABEL: &str = "xianyu-site";

const GOOFISH_URL: &str = "https://www.goofish.com/";

/// 打开内嵌闲鱼窗口并注入 cookies。
pub fn open_xianyu_site(app: &tauri::AppHandle, account: &ChannelAccount) -> Result<(), String> {
    // 已存在则聚焦返回。
    if let Some(window) = app.get_webview_window(XIANYU_WEBVIEW_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let cookies = crate::channels::xianyu::cookies::parse_credential(&account.credential);
    if cookies.is_empty() {
        return Err("账号凭据中没有 cookies".into());
    }

    let url = GOOFISH_URL
        .parse::<url::Url>()
        .map_err(|error| format!("闲鱼 URL 解析失败: {error}"))?;

    let builder = WebviewWindowBuilder::new(app, XIANYU_WEBVIEW_LABEL, WebviewUrl::External(url))
        .title(format!("闲鱼 — {}", account.name))
        .inner_size(980.0, 720.0)
        .on_navigation(|url| url.as_str().starts_with("https://www.goofish.com"));

    let window = builder
        .build()
        .map_err(|error| format!("创建闲鱼窗口失败: {error}"))?;

    // 注入 cookies（逐个，匹配 domain）。
    for cookie in cookies
        .iter()
        .filter(|cookie| cookie.domain.contains("goofish.com"))
    {
        if let Err(error) = inject_cookie(&window, cookie) {
            tracing::warn!(%error, cookie = %cookie.name, "注入 cookie 失败");
        }
    }

    Ok(())
}

/// 关闭内嵌闲鱼窗口。
pub fn close_xianyu_site(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(XIANYU_WEBVIEW_LABEL) {
        window
            .destroy()
            .map_err(|error| format!("关闭闲鱼窗口失败: {error}"))?;
    }
    Ok(())
}

fn inject_cookie(window: &WebviewWindow, cookie: &ChannelCookie) -> Result<(), String> {
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
    window
        .set_cookie(cookie)
        .map_err(|error| format!("set_cookie 失败: {error}"))
}
