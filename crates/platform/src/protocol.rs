//! 渠道统一协议 — 多平台实现的扩展点。
//!
//! 新平台接入：在 `crates/platform/src/<platform>/` 实现 [`ChannelProtocol`]，
//! 并注册进 [`super::dispatcher::ChannelDispatcher`]。

use async_trait::async_trait;
use common::DingDaResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

/// 渠道类型。新增平台时在此扩展（与 `capabilities::builtin_descriptors` 对齐）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelKind {
    Xianyu,
    /// 小红书（协议实现待接入，能力层已声明）。
    Xiaohongshu,
    /// 抖音（协议实现待接入，能力层已声明）。
    Douyin,
}

impl ChannelKind {
    /// 契约中的渠道标识（小写字符串）。
    pub fn as_str(&self) -> &'static str {
        match self {
            ChannelKind::Xianyu => "xianyu",
            ChannelKind::Xiaohongshu => "xiaohongshu",
            ChannelKind::Douyin => "douyin",
        }
    }

    /// 从契约字符串解析渠道类型；未知类型返回 `None`。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "xianyu" => Some(ChannelKind::Xianyu),
            "xiaohongshu" => Some(ChannelKind::Xiaohongshu),
            "douyin" => Some(ChannelKind::Douyin),
            _ => None,
        }
    }

    /// 全部已知平台。
    pub fn all() -> [Self; 3] {
        [
            ChannelKind::Xianyu,
            ChannelKind::Xiaohongshu,
            ChannelKind::Douyin,
        ]
    }
}

impl std::fmt::Display for ChannelKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// 连接状态枚举（与 UI 展示、`channel.event.status` 契约字符串对齐）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl ConnectionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionState::Disconnected => "disconnected",
            ConnectionState::Connecting => "connecting",
            ConnectionState::Connected => "connected",
            ConnectionState::Error => "error",
        }
    }

    #[allow(dead_code)]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "connecting" => ConnectionState::Connecting,
            "connected" => ConnectionState::Connected,
            "error" => ConnectionState::Error,
            _ => ConnectionState::Disconnected,
        }
    }
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// 渠道协议错误。
#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("channel error: {0}")]
    Protocol(String),
    #[error("not connected: {0}")]
    NotConnected(String),
    #[error("transport: {0}")]
    Transport(String),
}

/// 入站消息监听器 — 由应用层（coordinator）实现，协议层回调。
#[async_trait]
pub trait InboundListener: Send + Sync {
    async fn on_message(&self, message: ChannelInboundMessage);
    async fn on_state(&self, account_id: &str, state: ConnectionState, detail: Option<String>);
}

/// 协议层归一化后的入站消息（应用层继续加工为 `common::contracts::ChannelMessage`）。
#[derive(Debug, Clone)]
pub struct ChannelInboundMessage {
    pub account_id: String,
    pub peer_id: String,
    pub peer_name: String,
    pub item_id: String,
    pub content: String,
    #[allow(dead_code)]
    pub created_at_ms: i64,
}

/// 渠道协议统一接口 — 平台各自实现。
///
/// 设计约束：
/// - 协议层不感知业务（不读库、不决策回复、不调 LLM）；
/// - 入站消息通过 [`InboundListener`] 上抛，由 Rust 协调者统一处理；
/// - `send` 返回平台侧消息 id，作为幂等键。
#[async_trait]
pub trait ChannelProtocol: Send + Sync {
    fn kind(&self) -> ChannelKind;

    /// 建立长连接并开始收消息（异步任务在内部派生）。
    async fn connect(&self, account: &ChannelAccount) -> DingDaResult<()>;

    /// 断开连接。
    async fn disconnect(&self) -> DingDaResult<()>;

    /// 向 `peer_id` 发送文本，返回平台侧消息 id。
    async fn send(&self, peer_id: &str, text: &str) -> DingDaResult<String>;

    fn connection_state(&self) -> ConnectionState;

    fn set_inbound_listener(&self, listener: Arc<dyn InboundListener>);
}

/// 渠道账号（业务层复用的契约 DTO）。
pub use common::contracts::ChannelAccount;

/// 将 [`ChannelError`] 转为全局 [`common::DingDaError`]（协议层边界汇总）。
impl From<ChannelError> for common::DingDaError {
    fn from(err: ChannelError) -> Self {
        common::DingDaError::channel(err.to_string())
    }
}
