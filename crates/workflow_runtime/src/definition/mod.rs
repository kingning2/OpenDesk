//! 工作流静态定义（来自 React Flow / Contract）。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

mod edge;
mod node;
mod workflow_def;

pub use edge::EdgeSpec;
pub use node::{NodeSpec, NodeType, RetryPolicy, RetryStrategy};
pub use workflow_def::{RunPolicy, WorkflowDefinition};
