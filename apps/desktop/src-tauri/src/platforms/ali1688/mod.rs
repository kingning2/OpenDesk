//! 1688 平台壳层（`platform_ali1688`）。
//!
//! 账号扫码与 CRUD 走两站共用 [`crate::platforms::core`]，由
//! `core::bootstrap::register_business` 无条件注册。
//! 1688 专属 IPC 见 [`ipc`]；运行时装配收敛在 [`crate::platforms::runtime`]。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

pub mod ipc;
