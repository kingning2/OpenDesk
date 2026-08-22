//! 跨平台共享模块 — 闲鱼 / 1688 共用的账号与扫码逻辑。
//!
//! - [`account`] — 站点 Cookie 分袋、登录态判定、平台规范化（迁自 `business::account::dual_site`）
//! - [`account_qr`] — 扫码成功后从 Cookie 派生业务账号（迁自 Tauri 壳层纯逻辑）
//!
//! 本模块无平台分支：登录页 / 后置动作等平台差异由上层注入。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

pub mod account;
pub mod account_qr;

pub use account::{
    cookie_1688_looks_logged_in, cookie_domains_for_log, cookie_header_for_site,
    cookies_have_1688_unb, cookies_include_1688_domain, dual_site_login_remark,
    normalize_account_platform, SiteBag,
};
pub use account_qr::account_from_cookies;
