//! 共享账号站点辅助 — 闲鱼 + 1688 Cookie 分袋、登录态判定与平台规范化。
//!
//! 跨平台共享（两站共用），迁自 `business::account::dual_site`；
//! 仅依赖 `common::contracts::ChannelCookie`，无平台分支。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

use common::contracts::ChannelCookie;

/// 规范化平台标识。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-22
///
/// # 参数
///
/// * `platform` — 原始平台名
///
/// # 返回值
///
/// `xianyu` 或 `ali1688`。
pub fn normalize_account_platform(platform: &str) -> &'static str {
    match platform.trim().to_ascii_lowercase().as_str() {
        "ali1688" | "1688" | "alibaba1688" => "ali1688",
        _ => "xianyu",
    }
}

/// 根据双侧探活结果生成账号备注文案（不用「扫码登录」）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-22
///
/// # 参数
///
/// * `xianyu_ok` — 闲鱼登录态是否有效
/// * `ali1688_ok` — 1688 登录态是否有效
///
/// # 返回值
///
/// 备注字符串，例如 `闲鱼+1688已登录`。
pub fn dual_site_login_remark(xianyu_ok: bool, ali1688_ok: bool) -> String {
    match (xianyu_ok, ali1688_ok) {
        (true, true) => "闲鱼+1688已登录".to_string(),
        (true, false) => "仅闲鱼已登录".to_string(),
        (false, true) => "仅1688已登录".to_string(),
        (false, false) => "登录态未确认".to_string(),
    }
}

/// Cookie 分袋目标站点。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-22
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteBag {
    /// 闲鱼（goofish）+ 淘宝共享域。
    Xianyu,
    /// 1688 + 淘宝共享域。
    Ali1688,
}

/// 按域过滤后拼成 `name=value; …` Cookie 头。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-22
///
/// # 参数
///
/// * `cookies` — 含 domain 的 Cookie 列表
/// * `bag` — 目标站点袋
///
/// # 返回值
///
/// Cookie 请求头字符串。
pub fn cookie_header_for_site(cookies: &[ChannelCookie], bag: SiteBag) -> String {
    cookies
        .iter()
        .filter(|cookie| domain_matches_bag(&cookie.domain, bag))
        .map(|cookie| format!("{}={}", cookie.name, cookie.value))
        .collect::<Vec<_>>()
        .join("; ")
}

/// 是否导出了 1688 站域 Cookie。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-22
///
/// # 参数
///
/// * `cookies` — sidecar 导出列表
///
/// # 返回值
///
/// 任一 domain 含 `1688` 时为 `true`。
pub fn cookies_include_1688_domain(cookies: &[ChannelCookie]) -> bool {
    cookies
        .iter()
        .any(|cookie| cookie.domain.to_lowercase().contains("1688"))
}

/// 是否已有 1688 登录态（对齐 1688-cli：`unb` 且 domain 含 `1688.com`）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-22
///
/// # 参数
///
/// * `cookies` — sidecar 导出列表
///
/// # 返回值
///
/// 命中时为 `true`。淘宝域上的 `unb` 不算 1688 已登录。
pub fn cookies_have_1688_unb(cookies: &[ChannelCookie]) -> bool {
    cookies
        .iter()
        .any(|cookie| cookie.name == "unb" && cookie.domain.to_lowercase().contains("1688.com"))
}

/// Cookie 域名去重列表（不含值，供判定日志）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-22
///
/// # 参数
///
/// * `cookies` — sidecar 导出列表
///
/// # 返回值
///
/// 排序后的域名。
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

/// 1688 Cookie 串是否像已登录（含非空 `unb`）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-22
///
/// # 参数
///
/// * `cookie_str` — Cookie 原文
///
/// # 返回值
///
/// 含非空 `unb=` 时为 `true`。
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

fn domain_matches_bag(domain: &str, bag: SiteBag) -> bool {
    let d = domain.to_lowercase();
    let shared = d.contains("taobao")
        || d.contains("tmall")
        || d.contains("alipay")
        || d.contains("alibaba.com");
    match bag {
        SiteBag::Xianyu => d.contains("goofish") || shared,
        SiteBag::Ali1688 => d.contains("1688") || shared,
    }
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
        assert_eq!(normalize_account_platform("xianyu"), "xianyu");
        assert_eq!(normalize_account_platform("ali1688"), "ali1688");
        assert_eq!(normalize_account_platform("1688"), "ali1688");
        assert_eq!(normalize_account_platform(""), "xianyu");
    }

    #[test]
    fn dual_site_remark_variants() {
        assert_eq!(dual_site_login_remark(true, true), "闲鱼+1688已登录");
        assert_eq!(dual_site_login_remark(true, false), "仅闲鱼已登录");
        assert_eq!(dual_site_login_remark(false, true), "仅1688已登录");
        assert_eq!(dual_site_login_remark(false, false), "登录态未确认");
    }

    #[test]
    fn splits_cookies_by_domain() {
        let cookies = vec![
            cookie("unb", "U1", ".taobao.com"),
            cookie("_m_h5_tk", "xy", ".goofish.com"),
            cookie("_m_h5_tk", "ali", ".1688.com"),
            cookie("x5sec", "1", ".1688.com"),
        ];
        let xy = cookie_header_for_site(&cookies, SiteBag::Xianyu);
        let ali = cookie_header_for_site(&cookies, SiteBag::Ali1688);
        assert!(xy.contains("unb=U1"));
        assert!(xy.contains("_m_h5_tk=xy"));
        assert!(!xy.contains("x5sec=1"));
        assert!(ali.contains("unb=U1"));
        assert!(ali.contains("_m_h5_tk=ali"));
        assert!(ali.contains("x5sec=1"));
        assert!(!ali.contains("_m_h5_tk=xy"));
        assert!(cookies_include_1688_domain(&cookies));
        assert!(cookie_1688_looks_logged_in(&ali));
        assert!(cookies_have_1688_unb(&[cookie("unb", "U1", ".1688.com")]));
        assert!(!cookies_have_1688_unb(&[cookie(
            "unb",
            "U1",
            ".taobao.com"
        )]));
    }
}
