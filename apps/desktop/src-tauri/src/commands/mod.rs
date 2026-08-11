//! Tauri IPC command 按业务域分组。
//!
//! - [`agent`] — Agent / sidecar ping
//! - [`ai`] — AI 平台与账号配置
//! - [`license`] — 授权状态与激活
//!
//! 作者：coisini
//! 创建时间：2026-07-21

pub mod agent;
pub mod ai;
pub mod license;

pub use agent::agent_ping;
pub use ai::{ai_config_get, ai_config_set, ai_test_api_key};
pub use license::{license_activate, license_machine_code, license_status};
