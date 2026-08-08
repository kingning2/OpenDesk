//! Ready 集合计算。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use crate::dag::WorkflowGraph;
use crate::id::NodeId;
use crate::state::NodeState;
use std::collections::HashMap;

/// 计算当前可调度节点（稳定按 NodeId 排序）。
///
/// @author coisini
/// @created 2026-07-23
///
/// @param graph - 图
/// @param states - 节点状态
/// @returns Ready 节点 id 列表
pub fn compute_ready_nodes(
    graph: &WorkflowGraph,
    states: &HashMap<NodeId, NodeState>,
) -> Vec<NodeId> {
    let mut ready = Vec::new();
    for node_id in graph.nodes.keys() {
        let current = match states.get(node_id) {
            Some(state) => *state,
            None => NodeState::Pending,
        };
        match current {
            NodeState::Pending | NodeState::Blocked | NodeState::Ready => {}
            NodeState::Running
            | NodeState::RetryWaiting
            | NodeState::Succeeded
            | NodeState::Failed
            | NodeState::Skipped
            | NodeState::Cancelled => continue,
        }

        let preds = graph.predecessors_of(node_id);
        let mut all_ok = true;
        for pred in preds {
            let pred_state = match states.get(pred) {
                Some(state) => *state,
                None => NodeState::Pending,
            };
            match pred_state.satisfies_dependency() {
                true => {}
                false => {
                    all_ok = false;
                    break;
                }
            }
        }

        match all_ok {
            true => ready.push(node_id.clone()),
            false => {}
        }
    }
    ready.sort();
    ready
}
