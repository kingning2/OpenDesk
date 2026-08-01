//! 工作流事件枚举。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use crate::id::{InstanceId, NodeId};
use crate::state::{NodeState, WorkflowState};
use serde::{Deserialize, Serialize};

/// Runtime 领域事件（非魔法字符串 topic）。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum WorkflowEvent {
    /// 工作流开始。
    WorkflowStarted {
        instance_id: InstanceId,
        state: WorkflowState,
    },
    /// 工作流完成。
    WorkflowCompleted { instance_id: InstanceId },
    /// 工作流失败。
    WorkflowFailed {
        instance_id: InstanceId,
        message: String,
    },
    /// 工作流暂停。
    WorkflowPaused { instance_id: InstanceId },
    /// 工作流取消。
    WorkflowCancelled { instance_id: InstanceId },
    /// 节点开始。
    NodeStarted {
        instance_id: InstanceId,
        node_id: NodeId,
    },
    /// 节点成功。
    NodeCompleted {
        instance_id: InstanceId,
        node_id: NodeId,
        state: NodeState,
    },
    /// 节点失败。
    NodeFailed {
        instance_id: InstanceId,
        node_id: NodeId,
        message: String,
    },
    /// 已安排 Retry。
    NodeRetryScheduled {
        instance_id: InstanceId,
        node_id: NodeId,
        attempt: u32,
        next_at_ms: i64,
    },
    /// Context 变更。
    ContextChanged {
        instance_id: InstanceId,
        version: u64,
    },
}

impl WorkflowEvent {
    /// 事件稳定名（适配层映射 Tauri topic 时使用）。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns 事件名
    pub fn name(&self) -> &'static str {
        match self {
            Self::WorkflowStarted { .. } => "workflow_started",
            Self::WorkflowCompleted { .. } => "workflow_completed",
            Self::WorkflowFailed { .. } => "workflow_failed",
            Self::WorkflowPaused { .. } => "workflow_paused",
            Self::WorkflowCancelled { .. } => "workflow_cancelled",
            Self::NodeStarted { .. } => "node_started",
            Self::NodeCompleted { .. } => "node_completed",
            Self::NodeFailed { .. } => "node_failed",
            Self::NodeRetryScheduled { .. } => "node_retry_scheduled",
            Self::ContextChanged { .. } => "context_changed",
        }
    }
}
