//! 1688 账号域 — Cookie 分袋与扫码落库。

mod cookie;
mod qr;

pub use cookie::{
    cookie_1688_looks_logged_in, cookie_header_from_cookies, cookies_have_1688_unb,
    cookies_include_1688_domain, normalize_platform, resolve_account_platform,
};
pub use qr::account_from_cookies;
