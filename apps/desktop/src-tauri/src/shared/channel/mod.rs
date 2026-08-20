//! 通用渠道壳层编排 — 协调器 + 持久化 + 协议 re-export。
//!
//! 协议抽象与各平台实现在 `crates/platform`（不依赖 Tauri）。
//! 本目录只放 Tauri 依赖的壳层适配（协调器使用 `tauri::Emitter`）。
//! 无 Tauri 依赖的渠道业务层（`ChannelRepo`、`filter_reply`）已移至 `dingda-business::channel`。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

pub use platform::dispatcher;
pub use platform::protocol;

// 从 dingda-business re-export，保持 coordinator 路径不变
pub use app::channel::{conversation_id_for, filter_reply, inbound_to_message, ChannelRepo};

#[cfg(platform_xianyu)]
pub mod cookie_renew;
pub mod coordinator;
