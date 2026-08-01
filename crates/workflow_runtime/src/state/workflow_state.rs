//! 工作流实例状态。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use serde::{Deserialize, Serialize};
use std::fmt;

/// 工作流实例生命周期状态（落库用枚举名，禁止数字码）。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    /// 已创建尚未调度。
    Pending,
    /// 正在执行。
    Running,
    /// 正在排空以暂停。
    Pausing,
    /// 已暂停。
    Paused,
    /// 正在收尾为成功。
    Completing,
    /// 成功结束。
    Completed,
    /// 正在收尾为失败。
    Failing,
    /// 失败结束。
    Failed,
    /// 正在取消。
    Cancelling,
    /// 已取消。
    Cancelled,
}

impl WorkflowState {
    /// 是否为终态。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @returns 终态为 true
    pub fn is_terminal(self) -> bool {
        match self {
            Self::Completed | Self::Failed | Self::Cancelled => true,
            Self::Pending
            | Self::Running
            | Self::Pausing
            | Self::Paused
            | Self::Completing
            | Self::Failing
            | Self::Cancelling => false,
        }
    }

    /// 是否可被恢复扫描视为进行中。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @returns 可恢复为 true
    pub fn is_recoverable(self) -> bool {
        match self {
            Self::Running | Self::Pausing | Self::Failing | Self::Cancelling | Self::Paused => true,
            Self::Pending | Self::Completing | Self::Completed | Self::Failed | Self::Cancelled => {
                false
            }
        }
    }

    /// 解析落库字符串。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param raw - 枚举名（snake_case）
    /// @returns 解析结果
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "pausing" => Some(Self::Pausing),
            "paused" => Some(Self::Paused),
            "completing" => Some(Self::Completing),
            "completed" => Some(Self::Completed),
            "failing" => Some(Self::Failing),
            "failed" => Some(Self::Failed),
            "cancelling" => Some(Self::Cancelling),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    /// 落库用 snake_case 名。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @returns 稳定字符串
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Pausing => "pausing",
            Self::Paused => "paused",
            Self::Completing => "completing",
            Self::Completed => "completed",
            Self::Failing => "failing",
            Self::Failed => "failed",
            Self::Cancelling => "cancelling",
            Self::Cancelled => "cancelled",
        }
    }
}

impl fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
