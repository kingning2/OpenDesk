//! 闲鱼消息领域 — 对齐 goofish-cli `commands/message/`。
//!
//! - [`frames`]：出站帧构造（/reg、heartbeat、listUserMessages 等）
//! - [`history`]：历史消息解析（从 WS 响应体抽出）
//! - [`push`]：`syncPushPackage` 推包解析（会话列表补齐）
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-21

pub mod frames;
pub mod history;
pub mod push;

pub use frames::*;
pub use push::{parse_sync_push_package, PushBatch, PushedMessage, PushedSession};
