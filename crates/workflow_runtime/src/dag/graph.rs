//! 校验后的工作流图（邻接表）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use crate::definition::NodeSpec;
use crate::id::NodeId;
use std::collections::HashMap;

/// Scheduler 使用的有向无环图。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Clone)]
pub struct WorkflowGraph {
    /// 节点规格。
    pub nodes: HashMap<NodeId, NodeSpec>,
    /// 后继邻接表。
    pub successors: HashMap<NodeId, Vec<NodeId>>,
    /// 前驱邻接表。
    pub predecessors: HashMap<NodeId, Vec<NodeId>>,
    /// 唯一 Start。
    pub start_id: NodeId,
    /// 全部 End。
    pub end_ids: Vec<NodeId>,
    /// 边分支：source → (branch_key → targets)；默认边 key 为空串。
    pub branch_successors: HashMap<NodeId, HashMap<String, Vec<NodeId>>>,
}

impl WorkflowGraph {
    /// 取节点规格。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param id - 节点 id
    /// @returns 规格引用
    pub fn node(&self, id: &NodeId) -> Option<&NodeSpec> {
        self.nodes.get(id)
    }

    /// 前驱列表。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param id - 节点 id
    /// @returns 前驱切片
    pub fn predecessors_of(&self, id: &NodeId) -> &[NodeId] {
        match self.predecessors.get(id) {
            Some(list) => list.as_slice(),
            None => &[],
        }
    }

    /// 后继列表（所有分支合并）。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param id - 节点 id
    /// @returns 后继切片
    pub fn successors_of(&self, id: &NodeId) -> &[NodeId] {
        match self.successors.get(id) {
            Some(list) => list.as_slice(),
            None => &[],
        }
    }
}
