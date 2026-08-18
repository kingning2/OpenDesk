//! 全平台共用 IPC commands。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

pub mod agent;
pub mod ai;
pub mod channel;
pub mod license;
pub mod log;
pub mod platform;
pub mod response;

pub use agent::agent_ping;
pub use ai::{ai_config_get, ai_config_set, ai_test_api_key};
pub use channel::{
    channel_connect, channel_disconnect, channel_qr_cancel, channel_qr_check, channel_qr_start,
    channel_send, channel_state_get, channel_state_set,
};
pub use license::{license_activate, license_machine_code, license_status};
pub use log::{log_clear, log_recent, log_write};
pub use platform::platform_descriptors;
pub use response::IpcResponse;
