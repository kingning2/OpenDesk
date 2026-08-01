//! 工作流 / 节点 / 执行 / Retry 状态枚举与合法迁移。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

mod execution_state;
mod node_state;
mod retry_state;
mod transition;
mod workflow_state;

pub use execution_state::ExecutionState;
pub use node_state::NodeState;
pub use retry_state::RetryState;
pub use transition::{
    transition_node_state, transition_workflow_state, NodeTransition, WorkflowTransition,
};
pub use workflow_state::WorkflowState;
