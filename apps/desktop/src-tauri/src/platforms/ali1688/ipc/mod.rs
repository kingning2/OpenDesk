//! 1688 业务 IPC commands。
//!
//! 账号 CRUD / 扫码登录为两站共用，见 [`crate::platforms::core`]；
//! 本模块保留 1688 专属命令。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

mod chain;
mod search;

pub use search::ali1688_search;
