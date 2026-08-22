//! DingDa 桌面端纯 Rust 业务逻辑。
//!
//! 本 crate 收录所有**不依赖 Tauri** 的业务代码，供 `apps/desktop/src-tauri` 的
//! Tauri 壳层引用。分层原则：
//!
//! ```text
//! apps/desktop/src-tauri  ← Tauri 专属胶水（IPC 命令、状态注册、Builder）
//!         ↓ 依赖
//! business/               ← 纯 Rust 业务（本 crate）
//!         ↓ 依赖
//! crates/**               ← 跨平台 Rust 基础设施（不含 Tauri）
//! ```
//!
//! ## 模块一览
//!
//! | 模块 | 职责 |
//! |---|---|
//! | [`logging`] | 应用日志初始化（终端 + 内存环形缓冲） |
//! | [`timing`] | 异步耗时日志（显式 `#[timed]` 时启用） |
//! | [`config`] | 应用配置（AI JSON + 插件/OCR tessdata） |
//! | [`agent`] | PingAgent 业务逻辑 |
//! | [`channel`] | 渠道 SQLite 存储 + 安全过滤 |
//! | [`event_sink`] | EventBus → [`common::events::EventSink`] 适配 |
//! | [`account`] | 账号管理服务 |
//! | [`item`] | 商品管理服务 |
//! | [`order`] | 订单管理服务 |
//! | [`risk`] | 风控日志服务 |
//! | [`setting`] | 用户设置服务 |
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

#[macro_use]
extern crate tracing;

pub use common::DingDaResult;

// --- 原 business 模块 ---
pub mod channel;
pub mod config;
pub mod event_sink;
pub mod logging;
pub mod timing;

pub use event_sink::KernelEventSink;

// --- 原 crates/app 模块（业务层，渠道无关） ---
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
    MonitorResult, MonitorResultStore, MonitorService, MonitorTask, MonitorTaskStore,
};
pub use order::{DeliveryInfoUpdate, DeliveryMethod, Order, OrderService, OrderStatus, OrderStore};
pub use risk::{
    RiskConfig, RiskLogItem, RiskLogPage, RiskLogQuery, RiskService, RiskStore,
    RiskTodaySuccessRate,
};
pub use setting::{UserSettingService, UserSettingStore};
