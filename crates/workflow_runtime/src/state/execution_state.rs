//! Scheduler 内部执行相位（可不落库）。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use serde::{Deserialize, Serialize};

/// Scheduler 内部相位。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionState {
    /// 空闲。
    Idle,
    /// 计算 Ready。
    Scheduling,
    /// 派发 Executor。
    Dispatching,
    /// 等待执行完成。
    AwaitingExecutors,
    /// 写检查点。
    Persisting,
    /// 发事件。
    PublishingEvents,
    /// 已停止。
    Stopped,
}
