//! DAG Builder。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use super::graph::WorkflowGraph;
use super::validate::validate_and_build;
use crate::definition::WorkflowDefinition;
use crate::error::WorkflowError;

/// 将 Workflow JSON 定义构建为邻接表图。
///
/// @author coisini
/// @created 2026-07-23
pub struct DagBuilder;

impl DagBuilder {
    /// 校验并构建图。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @param definition - 静态定义
    /// @returns 合法 WorkflowGraph
    pub fn build(definition: &WorkflowDefinition) -> Result<WorkflowGraph, WorkflowError> {
        let parts = validate_and_build(definition)?;
        Ok(WorkflowGraph {
            nodes: parts.nodes,
            successors: parts.successors,
            predecessors: parts.predecessors,
            start_id: parts.start_id,
            end_ids: parts.end_ids,
            branch_successors: parts.branch_successors,
        })
    }
}
