//! 节点类型、规格与 Retry 策略。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use crate::id::NodeId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::time::Duration;

/// 节点类型（封闭枚举 + Registry 键）。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// 图唯一入口。
    Start,
    /// 图出口。
    End,
    /// 延迟。
    Delay,
    /// 条件分支。
    If,
    /// 多路分支。
    Switch,
    /// HTTP 请求。
    Http,
    /// AI / LLM。
    Ai,
    /// 采集：生成关键词。
    CrawlerGenerate,
    /// 采集：搜索。
    CrawlerSearch,
    /// 采集：汇总。
    CrawlerSummary,
}

impl NodeType {
    /// 稳定字符串键。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns snake_case 名
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
            Self::Delay => "delay",
            Self::If => "if",
            Self::Switch => "switch",
            Self::Http => "http",
            Self::Ai => "ai",
            Self::CrawlerGenerate => "crawler_generate",
            Self::CrawlerSearch => "crawler_search",
            Self::CrawlerSummary => "crawler_summary",
        }
    }

    /// 解析字符串。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @param raw - 类型名
    /// @returns 解析结果
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "start" => Some(Self::Start),
            "end" => Some(Self::End),
            "delay" => Some(Self::Delay),
            "if" => Some(Self::If),
            "switch" => Some(Self::Switch),
            "http" => Some(Self::Http),
            "ai" => Some(Self::Ai),
            "crawler_generate" => Some(Self::CrawlerGenerate),
            "crawler_search" => Some(Self::CrawlerSearch),
            "crawler_summary" => Some(Self::CrawlerSummary),
            _ => None,
        }
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Retry 退避策略。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryStrategy {
    /// 立即重试。
    Immediate,
    /// 固定延迟。
    FixedDelay,
    /// 指数退避。
    ExponentialBackoff,
}

impl RetryStrategy {
    /// 计算下次等待时长。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @param attempt - 即将进行的 attempt（从 1 起）
    /// @param base - 基础延迟
    /// @returns 等待时长
    pub fn delay_for(self, attempt: u32, base: Duration) -> Duration {
        match self {
            Self::Immediate => Duration::from_millis(0),
            Self::FixedDelay => base,
            Self::ExponentialBackoff => {
                let factor = match 1u32.checked_shl(attempt.saturating_sub(1)) {
                    Some(value) => value,
                    None => u32::MAX,
                };
                base.saturating_mul(factor)
            }
        }
    }
}

/// 节点 Retry 策略配置。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// 最大重试次数（不含首次）。
    pub max_retry: u32,
    /// 策略。
    pub strategy: RetryStrategy,
    /// 基础延迟毫秒。
    pub base_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retry: 0,
            strategy: RetryStrategy::Immediate,
            base_delay_ms: 0,
        }
    }
}

impl RetryPolicy {
    /// 基础延迟。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns Duration
    pub fn base_delay(&self) -> Duration {
        Duration::from_millis(self.base_delay_ms)
    }
}

/// 静态节点规格。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeSpec {
    /// 节点 id。
    pub id: NodeId,
    /// 类型。
    pub node_type: NodeType,
    /// 节点配置 JSON。
    pub config: Value,
    /// Retry。
    pub retry: RetryPolicy,
}
