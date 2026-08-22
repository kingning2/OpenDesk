//! 1688 账号 Cookie 与平台标识辅助。
//!
//! 平台识别（`normalize_platform` / `resolve_account_platform`）为纯字符串识别，
//! 已下沉到共享底座 `platform-core`（两站共用），此处 re-export 保持
//! `platform_ali1688::resolve_account_platform` 路径可用。

pub use crate::shared::account::{normalize_platform, resolve_account_platform};

use common::contracts::ChannelCookie;

/// 按 1688 域过滤后拼成 `name=value; …` Cookie 头。
pub fn cookie_header_from_cookies(cookies: &[ChannelCookie]) -> String {
    cookies
        .iter()
        .filter(|cookie| domain_matches_1688(&cookie.domain))
        .map(|cookie| format!("{}={}", cookie.name, cookie.value))
        .collect::<Vec<_>>()
        .join("; ")
}

/// 是否导出了 1688 站域 Cookie。
pub fn cookies_include_1688_domain(cookies: &[ChannelCookie]) -> bool {
    cookies
        .iter()
        .any(|cookie| cookie.domain.to_lowercase().contains("1688"))
}

/// 是否已有 1688 登录态（对齐 1688-cli：`unb` 且 domain 含 `1688.com`）。
pub fn cookies_have_1688_unb(cookies: &[ChannelCookie]) -> bool {
    cookies
        .iter()
        .any(|cookie| cookie.name == "unb" && cookie.domain.to_lowercase().contains("1688.com"))
}

/// Cookie 串是否像已登录（含非空 `unb`）。
pub fn cookie_1688_looks_logged_in(cookie_str: &str) -> bool {
    let cookie = cookie_str.trim();
    if cookie.is_empty() {
        return false;
    }
    cookie.split(';').any(|part| {
        let part = part.trim();
        part.strip_prefix("unb=")
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
    })
}

fn domain_matches_1688(domain: &str) -> bool {
    let d = domain.to_lowercase();
    let shared = d.contains("taobao")
        || d.contains("tmall")
        || d.contains("alipay")
        || d.contains("alibaba.com");
    d.contains("1688") || shared
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
    fn builds_cookie_header_for_1688_domains() {
        let cookies = vec![
            cookie("unb", "U1", ".taobao.com"),
            cookie("_m_h5_tk", "xy", ".goofish.com"),
            cookie("_m_h5_tk", "ali", ".1688.com"),
            cookie("x5sec", "1", ".1688.com"),
        ];
        let header = cookie_header_from_cookies(&cookies);
        assert!(header.contains("unb=U1"));
        assert!(header.contains("_m_h5_tk=ali"));
        assert!(header.contains("x5sec=1"));
        assert!(!header.contains("_m_h5_tk=xy"));
        assert!(cookies_include_1688_domain(&cookies));
        assert!(cookie_1688_looks_logged_in(&header));
        assert!(cookies_have_1688_unb(&[cookie("unb", "U1", ".1688.com")]));
        assert!(!cookies_have_1688_unb(&[cookie(
            "unb",
            "U1",
            ".taobao.com"
        )]));
    }
}
