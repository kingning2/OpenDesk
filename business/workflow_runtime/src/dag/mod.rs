//! DAG：邻接表、Builder、合法性校验。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

mod builder;
mod graph;
mod validate;

pub use builder::DagBuilder;
pub use graph::WorkflowGraph;
