//! OpenDesk Workflow Runtime。
//!
//! DAG 调度、状态机、Executor Registry、检查点与恢复。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

// 状态机风格大量使用 match；与 clippy single_match / manual_unwrap_or 偏好冲突，在此统一允许。
#![allow(clippy::single_match)]
#![allow(clippy::manual_unwrap_or)]
#![allow(clippy::manual_unwrap_or_default)]

pub mod context;
pub mod dag;
pub mod definition;
pub mod error;
pub mod event;
pub mod executor;
pub mod id;
pub mod persistence;
pub mod recover;
pub mod runtime;
pub mod scheduler;
pub mod state;

pub use context::{ContextPatch, WorkflowContext};
pub use dag::{DagBuilder, WorkflowGraph};
pub use definition::{
    EdgeSpec, NodeSpec, NodeType, RetryPolicy, RetryStrategy, RunPolicy, WorkflowDefinition,
};
pub use error::WorkflowError;
pub use event::{InMemoryEventBus, WorkflowEvent, WorkflowEventBus};
pub use executor::{
    register_builtin_executors, ExecuteInput, ExecuteOutput, ExecutorRegistry, NodeExecutor,
};
pub use id::{InstanceId, NodeId, WorkflowId};
pub use persistence::{CheckpointGateway, InMemoryCheckpointStore};
pub use recover::RecoveryService;
pub use runtime::WorkflowRuntimeFacade;
pub use scheduler::{Scheduler, SchedulerConfig, SchedulerHandle};
pub use state::{
    transition_node_state, transition_workflow_state, ExecutionState, NodeState, NodeTransition,
    RetryState, WorkflowState, WorkflowTransition,
};
