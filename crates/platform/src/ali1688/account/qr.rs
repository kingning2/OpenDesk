//! 1688 扫码落库 — 从 sidecar Cookie 派生业务账号。

use super::cookie::cookie_header_from_cookies;
use crate::domain::account::{
    AccountAutomation, AccountStatus, DeliveryGuard, LoginMethod, ProxyConfig, XianyuAccount,
};
use crate::shared::account::cookie_domains_for_log;
use common::contracts::ChannelCookie;

/// 从 cookies 构造 1688 业务账号。
pub fn account_from_cookies(cookies: &[ChannelCookie]) -> XianyuAccount {
    let unb = cookies
        .iter()
        .find(|cookie| cookie.name == "unb")
        .map(|cookie| cookie.value.clone())
        .unwrap_or_default();

    let cookie = cookie_header_from_cookies(cookies);
    let account_id = if unb.is_empty() {
        "1688-qr".to_string()
    } else {
        format!("1688:{unb}")
    };
    let domains = cookie_domains_for_log(cookies).join(",");

    tracing::info!(platform = "ali1688", domains, "单站登录态已判定");

    XianyuAccount {
        id: 0,
        owner_id: 1,
        account_id,
        display_name: String::new(),
        avatar_url: String::new(),
        login_id: String::new(),
        login_password: String::new(),
        unb,
        cookie,
        cookie_1688: String::new(),
        platform: crate::ali1688::PLATFORM_ID.to_string(),
        login_method: LoginMethod::Qr,
        status: AccountStatus::Active,
        remark: String::new(),
        pause_duration_minutes: 10,
        last_login_at: Some(now_string()),
        last_refresh_at: None,
        proxy: ProxyConfig::default(),
        automation: AccountAutomation::default(),
        delivery_guard: DeliveryGuard::default(),
    }
}

fn now_string() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
