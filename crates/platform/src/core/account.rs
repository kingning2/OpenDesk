//! 共享账号站点辅助 — 闲鱼 Cookie 分袋与平台规范化入口。
//!
//! 跨平台共享（两站共用）；1688 专有逻辑见 [`crate::ali1688`].
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

use common::contracts::ChannelCookie;

/// 规范化平台标识（QR / IPC 路由用）。
///
/// 1688 变体委托 [`crate::ali1688::normalize_platform`]，其余默认闲鱼。
pub fn normalize_account_platform(platform: &str) -> &'static str {
    #[cfg(platform_ali1688)]
    {
        if let Some(normalized) = crate::ali1688::normalize_platform(platform) {
            return normalized;
        }
    }
    #[cfg(not(platform_ali1688))]
    let _ = platform;
    "xianyu"
}

/// 根据双侧探活结果生成账号备注文案（不用「扫码登录」）。
pub fn dual_site_login_remark(xianyu_ok: bool, ali1688_ok: bool) -> String {
    match (xianyu_ok, ali1688_ok) {
        (true, true) => "闲鱼+1688已登录".to_string(),
        (true, false) => "仅闲鱼已登录".to_string(),
        (false, true) => "仅1688已登录".to_string(),
        (false, false) => "登录态未确认".to_string(),
    }
}

/// 按闲鱼域过滤后拼成 `name=value; …` Cookie 头。
pub fn xianyu_cookie_header(cookies: &[ChannelCookie]) -> String {
    cookies
        .iter()
        .filter(|cookie| domain_matches_xianyu(&cookie.domain))
        .map(|cookie| format!("{}={}", cookie.name, cookie.value))
        .collect::<Vec<_>>()
        .join("; ")
}

/// Cookie 域名去重列表（不含值，供判定日志）。
pub fn cookie_domains_for_log(cookies: &[ChannelCookie]) -> Vec<String> {
    let mut domains: Vec<String> = cookies
        .iter()
        .map(|cookie| cookie.domain.clone())
        .filter(|domain| !domain.is_empty())
        .collect();
    domains.sort();
    domains.dedup();
    domains
}

fn domain_matches_xianyu(domain: &str) -> bool {
    let d = domain.to_lowercase();
    let shared = d.contains("taobao")
        || d.contains("tmall")
        || d.contains("alipay")
        || d.contains("alibaba.com");
    d.contains("goofish") || shared
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cookie(name: &str, value: &str, domain: &str) -> ChannelCookie {
        ChannelCookie {
            name: name.to_string(),
            value: value.to_string(),
            domain: domain.to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: None,
            secure: None,
            same_site: None,
        }
    }

    #[test]
    fn normalize_defaults_to_xianyu() {
        assert_eq!(normalize_account_platform("xianyu"), "xianyu");
        assert_eq!(normalize_account_platform(""), "xianyu");
    }

    #[cfg(platform_ali1688)]
    #[test]
    fn normalize_recognizes_1688_variants() {
        assert_eq!(normalize_account_platform("ali1688"), "ali1688");
        assert_eq!(normalize_account_platform("1688"), "ali1688");
    }

    #[test]
    fn dual_site_remark_variants() {
        assert_eq!(dual_site_login_remark(true, true), "闲鱼+1688已登录");
        assert_eq!(dual_site_login_remark(true, false), "仅闲鱼已登录");
        assert_eq!(dual_site_login_remark(false, true), "仅1688已登录");
        assert_eq!(dual_site_login_remark(false, false), "登录态未确认");
    }

    #[test]
    fn builds_xianyu_cookie_header() {
        let cookies = vec![
            cookie("unb", "U1", ".taobao.com"),
            cookie("_m_h5_tk", "xy", ".goofish.com"),
            cookie("_m_h5_tk", "ali", ".1688.com"),
            cookie("x5sec", "1", ".1688.com"),
        ];
        let xy = xianyu_cookie_header(&cookies);
        assert!(xy.contains("unb=U1"));
        assert!(xy.contains("_m_h5_tk=xy"));
        assert!(!xy.contains("x5sec=1"));
        assert!(!xy.contains("_m_h5_tk=ali"));
    }
}
