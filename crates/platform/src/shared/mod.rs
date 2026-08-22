//! platform-core crate — 渠道 Provider 共享层（两站共用）。
//!
//! 职责：
//! - `account` — 共享账号站点辅助（平台规范化、Cookie 分袋、登录备注）
//! - `account_qr` — 从 sidecar Cookie 构造业务账号（闲鱼/兜底构建器）
//! - `cookie` / `cookies` — Cookie 解析、设备 id、凭据形态转换工具
//! - `db` — 业务 SQLite 基础设施（`SqliteBusinessDb`，通用 JSON 记录表）
//! - `stores` — 各业务域存储适配器（`InMemory*Store`，实现 `business` 的 Store Ports）
//!
//! 本 crate 是渠道 Provider（`platform-xianyu` / `platform-ali1688`）的**共享底座**：
//! 两站复用账号库与 Cookie 工具，避免 1688 依赖 "xianyu" crate。
//! 不依赖 `protocol` seam，也不依赖任何具体平台。

pub mod account;
pub mod account_qr;
pub mod cookie;
pub mod cookies;
pub mod db;
pub mod stores;

pub use account::{
    cookie_domains_for_log, dual_site_login_remark, normalize_account_platform, normalize_platform,
    resolve_account_platform, xianyu_cookie_header,
};
pub use account_qr::account_from_cookies;
pub use db::SqliteBusinessDb;
pub use stores::{
    InMemoryAccountStore, InMemoryItemStore, InMemoryMonitorResultStore, InMemoryMonitorRunStore,
    InMemoryMonitorTaskStore, InMemoryOrderStore, InMemoryRiskStore, InMemoryUserSettingStore,
};
