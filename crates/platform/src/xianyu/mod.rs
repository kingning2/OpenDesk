//! platform-xianyu crate — 闲鱼渠道协议 **Provider**。
//!
//! 实现 `crate::protocol::ChannelProtocol`（[`XianyuChannel`]）。协议细节
//! （WS 握手、签名、编解码、收发帧）全部收敛在本 crate；Cookie 工具与
//! 业务 SQLite 数据层在共享底座 `platform-core`。
//!
//! 目录对齐 goofish-cli：`core/`（协议内核）+ 领域模块（`message` / `item` / `profile`）。
//!
//! 精简说明：发布 / 评价子页已下线，对应网关（`publish` / `rate`）一并删除。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-21

pub mod core;
pub mod item;
pub mod message;
pub mod profile;

/// 稳定 re-export：历史路径 `platform_xianyu::api` 等仍可用。
pub use core::api;
pub use core::dev_tunnel;
pub use core::mtop;
pub use core::mtop::{MtopClient, MtopRequest, MtopResponse};
pub use core::risk::{extract_punish_url, is_risk_control_text};
pub use core::session::{fetch_sessions, fetch_unread_sessions, SessionSummary};
pub use core::ws::XianyuChannel;
pub use item::{fetch_item_detail, fetch_seller_items, PlatformItem, PlatformItemDetail};
pub use profile::{fetch_message_headinfo, fetch_user_profile, UserProfile};
