//! platform crate — 多渠道协议抽象层。
//!
//! 职责：
//! - `protocol` — 渠道统一 `ChannelProtocol` trait + 入站消息归一化
//! - `dispatcher` — 多账号调度器（注册表 + 生命周期）
//! - `capabilities` — 平台能力清单（前端动态路由元数据）
//! - `registry` — 平台注册表（能力层 + 协议层统一入口）
//! - `xianyu` — 闲鱼协议实现（首个平台）
//!
//! 设计约束：
//! - **不依赖 Tauri**：本 crate 可被桌面壳、CLI、未来 Web 服务复用。
//! - 协议层不感知业务：不读库、不决策回复、不调 LLM。
//! - 新平台接入三步：`ChannelKind` 加枚举 → `builtin_descriptors` 声明能力 →
//!   在 `xianyu/` 同构子目录实现 [`protocol::ChannelProtocol`] 并注册进
//!   [`dispatcher::ChannelDispatcher`] / [`registry::PlatformRegistry`]。

#[macro_use]
extern crate tracing;

pub mod capabilities;
pub mod compile;
pub mod dispatcher;
pub mod protocol;
pub mod registry;
#[cfg(platform_xianyu)]
pub mod xianyu;

pub use capabilities::{PlatformCapabilities, PlatformCapability, PlatformDescriptor};
pub use common::DingDaResult;
pub use compile::{active_kind, is_active, is_active_id, ACTIVE_PLATFORM};
pub use dispatcher::{ChannelDispatcher, ChannelProtocolFactory, DispatcherError};
pub use protocol::{
    ChannelError, ChannelInboundMessage, ChannelKind, ChannelProtocol, ConnectionState,
    InboundListener,
};
pub use registry::{PlatformInfo, PlatformRegistry};
#[cfg(platform_xianyu)]
pub use xianyu::XianyuChannel;
