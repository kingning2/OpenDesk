//! 节点实例状态。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use serde::{Deserialize, Serialize};
use std::fmt;

/// 节点运行状态。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeState {
    /// 初始。
    Pending,
    /// 前驱未满足。
    Blocked,
    /// 可调度。
    Ready,
    /// 执行中。
    Running,
    /// 等待 Retry 延迟。
    RetryWaiting,
    /// 成功。
    Succeeded,
    /// 失败（重试耗尽或不可重试）。
    Failed,
    /// 分支未选中而跳过。
    Skipped,
    /// 随工作流取消。
    Cancelled,
}

impl NodeState {
    /// 前驱是否视为「已满足依赖」。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns Succeeded / Skipped 为 true
    pub fn satisfies_dependency(self) -> bool {
        match self {
            Self::Succeeded | Self::Skipped => true,
            Self::Pending
            | Self::Blocked
            | Self::Ready
            | Self::Running
            | Self::RetryWaiting
            | Self::Failed
            | Self::Cancelled => false,
        }
    }

    /// 是否为节点终态。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns 终态为 true
    pub fn is_terminal(self) -> bool {
        match self {
            Self::Succeeded | Self::Failed | Self::Skipped | Self::Cancelled => true,
            Self::Pending | Self::Blocked | Self::Ready | Self::Running | Self::RetryWaiting => {
                false
            }
        }
    }

    /// 解析落库字符串。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @param raw - snake_case 名
    /// @returns 解析结果
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "pending" => Some(Self::Pending),
            "blocked" => Some(Self::Blocked),
            "ready" => Some(Self::Ready),
            "running" => Some(Self::Running),
            "retry_waiting" => Some(Self::RetryWaiting),
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            "skipped" => Some(Self::Skipped),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    /// 落库用名。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns 稳定字符串
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Blocked => "blocked",
            Self::Ready => "ready",
            Self::Running => "running",
            Self::RetryWaiting => "retry_waiting",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
            Self::Cancelled => "cancelled",
        }
    }
}

impl fmt::Display for NodeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
