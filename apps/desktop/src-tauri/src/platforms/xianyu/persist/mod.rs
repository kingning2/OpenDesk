//! 闲鱼业务存储 — 从 `dingda-business::xianyu` 统一 re-export。
//!
//! 所有 SQLite 存储适配器已集中在 `business/src/xianyu/stores.rs` 单文件实现，
//! 此处不再按域拆分子模块，避免同一业务散落多文件。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

pub use app::xianyu::{
    InMemoryAccountStore, InMemoryAddressStore, InMemoryAutoReplyLogStore, InMemoryBatchStore,
    InMemoryBlacklistStore, InMemoryCardStore, InMemoryFeedbackStore, InMemoryFilterStore,
    InMemoryItemStore, InMemoryKeywordStore, InMemoryNotificationStore, InMemoryOrderStore,
    InMemoryPublishLogStore, InMemoryPublishMaterialStore, InMemoryRiskStore,
    InMemoryUserSettingStore,
};
