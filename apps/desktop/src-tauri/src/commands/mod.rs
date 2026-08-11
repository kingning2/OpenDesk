//! Tauri IPC command 按业务域分组。
//!
//! - [`agent`] — Agent / sidecar ping
//! - [`license`] — 授权状态与激活
//!
//! 作者：coisini
//! 创建时间：2026-07-21

pub mod agent;
pub mod license;

pub use agent::agent_ping;
pub use license::{license_activate, license_machine_code, license_status};
