//! platform-ali1688 crate — 1688 账号 **Provider**。
//!
//! 职责：
//! - `account` — Cookie 分袋、平台解析、扫码落库
//! - 浏览器任务（搜索 / 登录探针）由 Python sidecar 实现
//!
//! 共享底座（账号构造、Cookie 工具、业务数据层）在 `platform-core`。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

pub mod account;

pub use account::{
    account_from_cookies, cookie_1688_looks_logged_in, cookie_header_from_cookies,
    cookies_have_1688_unb, cookies_include_1688_domain, normalize_platform,
    resolve_account_platform,
};

/// 1688 平台标识（契约/编译期常量）。
pub const PLATFORM_ID: &str = "ali1688";
