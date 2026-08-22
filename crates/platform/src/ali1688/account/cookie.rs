//! 1688 账号 Cookie 与平台标识辅助。

use common::contracts::ChannelCookie;

/// 规范化 1688 平台标识；非 1688 变体返回 `None`。
pub fn normalize_platform(platform: &str) -> Option<&'static str> {
    match platform.trim().to_ascii_lowercase().as_str() {
        "ali1688" | "1688" | "alibaba1688" => Some("ali1688"),
        _ => None,
    }
}

/// 解析账号所属平台（对齐前端 [`resolveAccountPlatform`]）。
pub fn resolve_account_platform(account_id: &str, platform: &str) -> &'static str {
    if account_id.starts_with("1688:") {
        return "ali1688";
    }
    normalize_platform(platform).unwrap_or("xianyu")
}

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
    fn normalize_platform_variants() {
        assert_eq!(normalize_platform("ali1688"), Some("ali1688"));
        assert_eq!(normalize_platform("1688"), Some("ali1688"));
        assert_eq!(normalize_platform("xianyu"), None);
    }

    #[test]
    fn resolve_platform_prefers_1688_account_id_prefix() {
        assert_eq!(
            resolve_account_platform("1688:2200574208023", "xianyu"),
            "ali1688"
        );
        assert_eq!(resolve_account_platform("xy123", "ali1688"), "ali1688");
        assert_eq!(resolve_account_platform("xy123", ""), "xianyu");
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
