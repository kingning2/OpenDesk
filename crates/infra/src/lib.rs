//! infra crate — 基础设施（**自包含**）：进程内事件总线 + Python sidecar 运行时 + 基础设施适配器。
//!
//! 结构：
//! - `event` — 进程内 pub/sub 事件总线（`EventBus` / `EventHandler` / `InMemoryEventBus`）
//! - `sidecar` — Python sidecar 运行时（客户端 / 生命周期 / 日志管道 / 路由绑定）
//! - `agent_sidecar` — agent sidecar 网关适配器（实现 `ports::sidecar`）
//! - `license` — license 校验适配器（实现 `ports::license`）
//!
//! 只依赖 `common` / `ports`（共享叶子）+ 外部 crate，不依赖其它 DingDa crate。

#[macro_use]
extern crate tracing;

pub mod agent_sidecar;
pub mod event;
pub mod license;
pub mod sidecar;

pub use agent_sidecar::RuntimeAgentSidecar;
pub use event::{EventBus, EventHandler, InMemoryEventBus};
