//! 渠道业务层 — 多渠道客服。
//!
//! 目录说明（多平台扩展）：
//! - `protocol.rs` — 渠道统一 trait
//! - `dispatcher.rs` — 调度器
//! - `coordinator.rs` — 入站管线 + 自动回复决策
//! - `reply.rs` / `safety.rs` / `store.rs` — 决策 / 过滤 / 持久化
//! - `xianyu/` — 闲鱼协议实现（新平台加同构子目录）

pub mod commands;
pub mod coordinator;
pub mod dispatcher;
pub mod protocol;
pub mod reply;
pub mod safety;
pub mod store;
pub mod webview;
pub mod xianyu;
