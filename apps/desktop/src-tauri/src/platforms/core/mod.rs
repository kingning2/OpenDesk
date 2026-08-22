//! 多站共用 Tauri IPC（账号 CRUD / 扫码登录）+ 共用启动。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

pub mod account;
pub mod account_qr;
pub mod bootstrap;

pub use account::{
    account_create, account_delete, account_list, account_probe_login, account_set_status,
    account_update,
};
pub use account_qr::{account_qr_cancel, account_qr_check, account_qr_start};
