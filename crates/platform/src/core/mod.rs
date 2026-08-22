//! 跨平台共享模块 — 两站共用的账号 / 扫码逻辑（无 1688 专有实现）。
//!
//! - [`account`] — 闲鱼 Cookie 分袋、平台规范化入口
//! - [`account_qr`] — 扫码成功后从 Cookie 派生业务账号（1688 委托 [`crate::ali1688`]）
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

pub mod account;
pub mod account_qr;

pub use account::{
    cookie_domains_for_log, dual_site_login_remark, normalize_account_platform,
    xianyu_cookie_header,
};
pub use account_qr::account_from_cookies;
