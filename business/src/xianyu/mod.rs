//! 闲鱼业务模块 — SQLite 数据层 + 发布网关。
//!
//! 包含：
//! - [`db`] — `SqliteBusinessDb`：通用 JSON 记录 SQLite 基础设施
//! - [`stores`] — 所有业务域存储适配器
//! - [`publish`] — 发布网关内存实现
//! - [`rate`] — 评价网关（mtop）
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

pub mod db;
pub mod publish;
pub mod rate;
pub mod stores;

pub use db::SqliteBusinessDb;
pub use publish::InMemoryPublishGateway;
pub use rate::MtopRateGateway;
pub use stores::{
    append_log, update_log, InMemoryAccountStore, InMemoryAddressStore, InMemoryAutoReplyLogStore,
    InMemoryBatchStore, InMemoryBlacklistStore, InMemoryCardStore, InMemoryFeedbackStore,
    InMemoryFilterStore, InMemoryItemStore, InMemoryKeywordStore, InMemoryNotificationStore,
    InMemoryOrderStore, InMemoryPublishLogStore, InMemoryPublishMaterialStore, InMemoryRiskStore,
    InMemoryUserSettingStore,
};
