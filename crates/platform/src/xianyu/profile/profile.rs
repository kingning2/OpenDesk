//! 闲鱼用户资料拉取 — 连接后同步昵称与头像。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-19

use common::DingDaResult;

use crate::xianyu::core::mtop::{MtopClient, MtopRequest};

/// 闲鱼用户公开资料（导航接口 `mtop.idle.web.user.page.nav`）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
#[derive(Debug, Clone, Default)]
pub struct UserProfile {
    /// 展示昵称。
    pub display_name: String,
    /// 头像 URL。
    pub avatar_url: String,
}

/// 拉取当前登录用户的昵称与头像。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
///
/// # 参数
///
/// * `cookie_str` — 账号 Cookie 原文
///
/// # 返回值
///
/// 成功返回 `(UserProfile, 最新 Cookie)`；mtop 可能通过 set-cookie 刷新签名 token。
pub async fn fetch_user_profile(cookie_str: &str) -> DingDaResult<(UserProfile, String)> {
    let client = MtopClient::new(cookie_str)?;
    let request = MtopRequest::new("mtop.idle.web.user.page.nav", "1.0", serde_json::json!({}));
    let response = client.call(&request).await?;
    if !response.success() {
        return Err(format!("用户资料接口未成功: {}", response.ret).into());
    }

    let data = response.data().cloned().unwrap_or(serde_json::json!({}));
    let display_name = data
        .pointer("/module/base/displayName")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    let avatar_url = data
        .pointer("/module/base/avatar")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();

    let updated_cookie = client.cookie().await;
    Ok((
        UserProfile {
            display_name,
            avatar_url,
        },
        updated_cookie,
    ))
}
