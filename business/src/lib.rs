//! business crate — 桌面端应用胶水（不依赖 Tauri，纯 Rust）。
//!
//! 领域层（业务模型 + Store Ports + 领域服务）已下沉到 `crates/platform`；
//! 本 crate 只保留**应用壳胶水**：日志、配置、渠道存储、事件桥、耗时计时。
//! 领域逻辑从 `crates/platform` 导入。
//!
//! 分层：
//!
//! ```text
//! apps/desktop/src-tauri → business（胶水） + crates/platform（领域）
//! ```

#[macro_use]
extern crate tracing;

pub use common::DingDaResult;

pub mod channel;
pub mod config;
pub mod event_sink;
pub mod logging;
pub mod timing;

pub use event_sink::KernelEventSink;
