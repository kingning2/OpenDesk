//! 共享扫码账号派生 — 从 sidecar 导出的 Cookie 构造业务账号（按平台分袋）。
//!
//! 纯逻辑、无 Tauri 类型；Tauri 壳层在扫码成功后调用 [`account_from_cookies`] 落库。
//! 跨平台共享（闲鱼 / 1688 共用）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

use super::account::{
    cookie_domains_for_log, cookie_header_for_site, cookies_have_1688_unb, SiteBag,
};
use business::account::{
    AccountAutomation, AccountStatus, DeliveryGuard, LoginMethod, ProxyConfig, XianyuAccount,
};
use common::contracts::ChannelCookie;

/// 从 cookies 构造业务账号（单站）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-22
///
/// # 参数
///
/// * `platform` — `xianyu` / `ali1688`
/// * `cookies` — sidecar 导出的全 jar
///
/// # 返回值
///
/// 待落库的 [`XianyuAccount`]。
pub fn account_from_cookies(platform: &str, cookies: &[ChannelCookie]) -> XianyuAccount {
    let unb = cookies
        .iter()
        .find(|cookie| cookie.name == "unb")
        .map(|cookie| cookie.value.clone())
        .unwrap_or_default();

    let domains = cookie_domains_for_log(cookies).join(",");
    let (cookie, cookie_1688, remark, account_id) = if platform == "ali1688" {
        let cookie = cookie_header_for_site(cookies, SiteBag::Ali1688);
        let ok = cookies_have_1688_unb(cookies) || !cookie.trim().is_empty();
        let remark = if ok {
            "1688已登录".to_string()
        } else {
            "1688登录态未确认".to_string()
        };
        let account_id = if unb.is_empty() {
            "1688-qr".to_string()
        } else {
            format!("1688:{unb}")
        };
        (cookie, String::new(), remark, account_id)
    } else {
        let cookie = cookie_header_for_site(cookies, SiteBag::Xianyu);
        let ok = !cookie.trim().is_empty() && !unb.trim().is_empty();
        let remark = if ok {
            "闲鱼已登录".to_string()
        } else {
            "闲鱼登录态未确认".to_string()
        };
        let account_id = if unb.is_empty() {
            "xianyu-qr".to_string()
        } else {
            unb.clone()
        };
        (cookie, String::new(), remark, account_id)
    };

    tracing::info!(platform, %remark, domains, "单站登录态已判定");

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
        cookie_1688,
        platform: platform.to_string(),
        login_method: LoginMethod::Qr,
        status: AccountStatus::Active,
        remark,
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
