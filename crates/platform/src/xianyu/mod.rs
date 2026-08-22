//! 闲鱼渠道协议实现。
//!
//! 首个平台接入：协议细节（WS 握手、签名、编解码、收发帧）全部收敛在本目录，
//! 对上层只暴露 [`XianyuChannel`]（实现 [`super::protocol::ChannelProtocol`]）。
//!
//! 目录对齐 goofish-cli：`core/`（协议内核）+ 领域模块（`message` / `item` / `profile`）。
//! 数据层（`db` / `stores`）为闲鱼业务实现，迁自 `business/src/xianyu`，
//! 让平台目录真正自包含。
//!
//! 精简说明：发布 / 评价子页已下线，对应网关（`publish` / `rate`）一并删除。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-21

pub mod core;
pub mod db;
pub mod item;
pub mod message;
pub mod profile;
pub mod stores;

/// 稳定 re-export：历史路径 `platform::xianyu::api` 等仍可用。
pub use core::api;
pub use core::cookies;
pub use core::dev_tunnel;
pub use core::mtop;
pub use core::mtop::{MtopClient, MtopRequest, MtopResponse};
pub use core::risk::{extract_punish_url, is_risk_control_text};
pub use core::session::{fetch_sessions, fetch_unread_sessions, SessionSummary};
pub use core::ws::XianyuChannel;
pub use db::SqliteBusinessDb;
pub use item::{fetch_item_detail, fetch_seller_items, PlatformItem, PlatformItemDetail};
pub use profile::{fetch_message_headinfo, fetch_user_profile, UserProfile};
pub use stores::{
    InMemoryAccountStore, InMemoryItemStore, InMemoryMonitorResultStore, InMemoryMonitorRunStore,
    InMemoryMonitorTaskStore, InMemoryOrderStore, InMemoryRiskStore, InMemoryUserSettingStore,
};
