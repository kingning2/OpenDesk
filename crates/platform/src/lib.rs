//! platform crate — 渠道平台（**自包含**）：协议 seam + 领域层 + 共享底座 + 平台 Provider + SQLite 存储。
//!
//! 结构（各模块根 re-export 对齐其历史 crate 的公开 API）：
//! - `protocol` — 渠道协议 seam（`ChannelProtocol` + dispatcher + 能力清单 + 编译期平台选择）
//! - `domain` — 领域层（业务模型 + Store Ports + 领域服务）
//! - `shared` — Provider 共享底座（账号派生 / Cookie 工具 / 业务 SQLite 数据层）
//! - `xianyu` — 闲鱼协议 Provider（`#[cfg(platform_xianyu)]`）
//! - `ali1688` — 1688 账号 Provider（`#[cfg(platform_ali1688)]`）
//! - `storage` — 通用 SQLite 记录存储（`SqliteDb` + `RecordStore`）
//!
//! 只依赖 `common` / `ports`（共享叶子）+ 外部 crate，不依赖其它 DingDa crate。

#[macro_use]
extern crate tracing;

#[cfg(platform_ali1688)]
pub mod ali1688;
pub mod domain;
pub mod protocol;
pub mod shared;
pub mod storage;
#[cfg(platform_xianyu)]
pub mod xianyu;

pub use protocol::{
    ChannelAccount, ChannelDispatcher, ChannelError, ChannelInboundMessage, ChannelKind,
    ChannelProtocol, ChannelProtocolFactory, ConnectionState, ConversationSync, DispatcherError,
    HistoryMessage, InboundListener, PlatformCapabilities, PlatformCapability, PlatformDescriptor,
    PlatformInfo, PlatformRegistry,
};
pub use shared::{
    normalize_account_platform, resolve_account_platform, InMemoryAccountStore, InMemoryItemStore,
    InMemoryMonitorResultStore, InMemoryMonitorRunStore, InMemoryMonitorTaskStore,
    InMemoryOrderStore, InMemoryRiskStore, InMemoryUserSettingStore, SqliteBusinessDb,
};
