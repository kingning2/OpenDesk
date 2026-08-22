//! 闲鱼协议核心 — 对齐 goofish-cli `core/`。
//!
//! 负责 mtop / HTTP / 签名 / 会话 / WebSocket / 开发帧隧道。
//! Cookie 工具（`cookie` / `cookies`）已下沉到 `platform-core`。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-21

pub mod api;
pub mod codec;
pub mod dev_tunnel;
pub mod http;
pub mod mtop;
pub mod risk;
pub mod session;
pub mod sign;
pub mod ws;
