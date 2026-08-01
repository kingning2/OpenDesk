//! 状态迁移（仅 match，禁止 if status ==）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use super::{NodeState, WorkflowState};
use crate::error::WorkflowError;

/// 工作流目标迁移意图。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowTransition {
    /// 开始运行。
    Start,
    /// 请求暂停。
    RequestPause,
    /// 暂停完成。
    PauseDrained,
    /// 从暂停恢复。
    Resume,
    /// 开始成功收尾。
    BeginComplete,
    /// 成功收尾完成。
    FinishComplete,
    /// 开始失败收尾。
    BeginFail,
    /// 失败收尾完成。
    FinishFail,
    /// 请求取消。
    RequestCancel,
    /// 取消完成。
    FinishCancel,
}

/// 节点目标迁移意图。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeTransition {
    /// 前驱未满足。
    Block,
    /// 变为可调度。
    MakeReady,
    /// 开始执行。
    Dispatch,
    /// 执行成功。
    Succeed,
    /// 进入 Retry 等待。
    ScheduleRetry,
    /// 失败终态。
    Fail,
    /// 分支跳过。
    Skip,
    /// 取消。
    Cancel,
    /// 人工重试：Failed → Ready。
    ManualRetry,
}

/// 应用工作流状态迁移。
///
/// @author Xiaoman
/// @created 2026-07-23
///
/// @param current - 当前状态
/// @param transition - 意图
/// @returns 新状态或非法迁移错误
pub fn transition_workflow_state(
    current: WorkflowState,
    transition: WorkflowTransition,
) -> Result<WorkflowState, WorkflowError> {
    let next = match (current, transition) {
        (WorkflowState::Pending, WorkflowTransition::Start) => WorkflowState::Running,
        (WorkflowState::Running, WorkflowTransition::RequestPause) => WorkflowState::Pausing,
        (WorkflowState::Pausing, WorkflowTransition::PauseDrained) => WorkflowState::Paused,
        (WorkflowState::Paused, WorkflowTransition::Resume) => WorkflowState::Running,
        (WorkflowState::Running, WorkflowTransition::BeginComplete) => WorkflowState::Completing,
        (WorkflowState::Completing, WorkflowTransition::FinishComplete) => WorkflowState::Completed,
        (WorkflowState::Running, WorkflowTransition::BeginFail) => WorkflowState::Failing,
        (WorkflowState::Failing, WorkflowTransition::FinishFail) => WorkflowState::Failed,
        (WorkflowState::Running, WorkflowTransition::RequestCancel) => WorkflowState::Cancelling,
        (WorkflowState::Pausing, WorkflowTransition::RequestCancel) => WorkflowState::Cancelling,
        (WorkflowState::Paused, WorkflowTransition::RequestCancel) => WorkflowState::Cancelling,
        (WorkflowState::Cancelling, WorkflowTransition::FinishCancel) => WorkflowState::Cancelled,
        (from, _) => {
            return Err(WorkflowError::InvalidStateTransition {
                from: from.to_string(),
                to: format!("{transition:?}"),
            });
        }
    };
    Ok(next)
}

/// 应用节点状态迁移。
///
/// @author Xiaoman
/// @created 2026-07-23
///
/// @param current - 当前状态
/// @param transition - 意图
/// @returns 新状态或非法迁移错误
pub fn transition_node_state(
    current: NodeState,
    transition: NodeTransition,
) -> Result<NodeState, WorkflowError> {
    let next = match (current, transition) {
        (NodeState::Pending, NodeTransition::Block) => NodeState::Blocked,
        (NodeState::Pending, NodeTransition::MakeReady) => NodeState::Ready,
        (NodeState::Blocked, NodeTransition::MakeReady) => NodeState::Ready,
        (NodeState::Ready, NodeTransition::Dispatch) => NodeState::Running,
        (NodeState::Ready, NodeTransition::Skip) => NodeState::Skipped,
        (NodeState::Ready, NodeTransition::Cancel) => NodeState::Cancelled,
        (NodeState::Running, NodeTransition::Succeed) => NodeState::Succeeded,
        (NodeState::Running, NodeTransition::ScheduleRetry) => NodeState::RetryWaiting,
        (NodeState::Running, NodeTransition::Fail) => NodeState::Failed,
        (NodeState::Running, NodeTransition::Cancel) => NodeState::Cancelled,
        (NodeState::RetryWaiting, NodeTransition::MakeReady) => NodeState::Ready,
        (NodeState::RetryWaiting, NodeTransition::Cancel) => NodeState::Cancelled,
        (NodeState::Failed, NodeTransition::ManualRetry) => NodeState::Ready,
        (NodeState::Blocked, NodeTransition::Cancel) => NodeState::Cancelled,
        (NodeState::Pending, NodeTransition::Cancel) => NodeState::Cancelled,
        (from, _) => {
            return Err(WorkflowError::InvalidStateTransition {
                from: from.to_string(),
                to: format!("{transition:?}"),
            });
        }
    };
    Ok(next)
}
