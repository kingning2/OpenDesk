//! domain crate — 领域层（业务模型 + Store Ports + 领域服务）。
//!
//! 收录 `business` 原领域模块：`account` / `item` / `order` / `risk` / `setting` / `monitor`。
//! 本 crate **不依赖 Tauri、不依赖应用壳（`business` / `src-tauri`）**；
//! Provider 层（`platform-core`）实现这里定义的 Store Ports。
//!
//! ## 模块一览
//!
//! | 模块 | 职责 |
//! |---|---|
//! | [`account`] | 账号模型 + `AccountStore` Port + `AccountService` |
//! | [`item`] | 商品模型 + `ItemStore` Port + `ItemService` |
//! | [`order`] | 订单模型 + `OrderStore` Port + `OrderService` |
//! | [`risk`] | 风控日志模型 + `RiskStore` Port + `RiskService` |
//! | [`setting`] | 用户设置 + `UserSettingStore` Port + `UserSettingService` |
//! | [`monitor`] | 商品监控模型 + 3 个 Store Port + `MonitorService` |
//!
//! 分层：
//!
//! ```text
//! business / src-tauri → domain → crates/common
//! platform-core        → domain（实现 Store Ports，供 business Service 调用）
//! ```

pub use common::DingDaResult;

pub mod account;
pub mod item;
pub mod monitor;
pub mod order;
pub mod risk;
pub mod setting;

pub use account::{
    AccountAutomation, AccountService, AccountServiceError, AccountStatus, AccountStore,
    AccountUpdate, DeliveryGuard, LoginMethod, ProxyConfig, XianyuAccount,
};
pub use item::{Item, ItemQuery, ItemService, ItemStore};
pub use monitor::{
    MonitorResult, MonitorResultStore, MonitorRun, MonitorRunStore, MonitorService, MonitorTask,
    MonitorTaskStore,
};
pub use order::{DeliveryInfoUpdate, DeliveryMethod, Order, OrderService, OrderStatus, OrderStore};
pub use risk::{
    RiskConfig, RiskLogItem, RiskLogPage, RiskLogQuery, RiskService, RiskStore,
    RiskTodaySuccessRate,
};
pub use setting::{UserSettingService, UserSettingStore};
