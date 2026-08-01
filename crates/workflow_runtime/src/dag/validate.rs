//! DAG 合法性校验。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use crate::definition::{NodeType, WorkflowDefinition};
use crate::error::WorkflowError;
use crate::id::NodeId;
use std::collections::HashMap;

/// 图构建中间结构。
///
/// @author Xiaoman
/// @created 2026-07-23
pub(crate) struct GraphParts {
    pub nodes: HashMap<NodeId, crate::definition::NodeSpec>,
    pub successors: HashMap<NodeId, Vec<NodeId>>,
    pub predecessors: HashMap<NodeId, Vec<NodeId>>,
    pub branch_successors: HashMap<NodeId, HashMap<String, Vec<NodeId>>>,
    pub start_id: NodeId,
    pub end_ids: Vec<NodeId>,
}

/// 校验定义并组装邻接表。
///
/// @author Xiaoman
/// @created 2026-07-23
///
/// @param definition - 静态定义
/// @returns 组装结果或 InvalidGraph
pub(crate) fn validate_and_build(
    definition: &WorkflowDefinition,
) -> Result<GraphParts, WorkflowError> {
    let mut nodes: HashMap<NodeId, crate::definition::NodeSpec> = HashMap::new();
    for node in &definition.nodes {
        match nodes.insert(node.id.clone(), node.clone()) {
            None => {}
            Some(_) => {
                return Err(WorkflowError::invalid_graph(format!(
                    "duplicate node id: {}",
                    node.id
                )));
            }
        }
    }

    if nodes.is_empty() {
        return Err(WorkflowError::invalid_graph("workflow has no nodes"));
    }

    let mut successors: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    let mut predecessors: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    let mut branch_successors: HashMap<NodeId, HashMap<String, Vec<NodeId>>> = HashMap::new();

    for node_id in nodes.keys() {
        successors.insert(node_id.clone(), Vec::new());
        predecessors.insert(node_id.clone(), Vec::new());
        branch_successors.insert(node_id.clone(), HashMap::new());
    }

    for edge in &definition.edges {
        match nodes.contains_key(&edge.source) {
            true => {}
            false => {
                return Err(WorkflowError::invalid_graph(format!(
                    "edge {} references missing source {}",
                    edge.id, edge.source
                )));
            }
        }
        match nodes.contains_key(&edge.target) {
            true => {}
            false => {
                return Err(WorkflowError::invalid_graph(format!(
                    "edge {} references missing target {}",
                    edge.id, edge.target
                )));
            }
        }

        match nodes.get(&edge.source).map(|spec| spec.node_type) {
            Some(NodeType::End) => {
                return Err(WorkflowError::invalid_graph(format!(
                    "illegal edge from End node {}",
                    edge.source
                )));
            }
            _ => {}
        }

        if let Some(list) = successors.get_mut(&edge.source) {
            list.push(edge.target.clone());
        }
        if let Some(list) = predecessors.get_mut(&edge.target) {
            list.push(edge.source.clone());
        }

        let branch_key = match &edge.branch {
            Some(value) => value.clone(),
            None => String::new(),
        };
        if let Some(map) = branch_successors.get_mut(&edge.source) {
            map.entry(branch_key).or_default().push(edge.target.clone());
        }
    }

    let mut start_ids: Vec<NodeId> = nodes
        .iter()
        .filter_map(|(id, spec)| match spec.node_type {
            NodeType::Start => Some(id.clone()),
            _ => None,
        })
        .collect();
    start_ids.sort();

    match start_ids.len() {
        1 => {}
        0 => return Err(WorkflowError::invalid_graph("workflow has no Start node")),
        _ => {
            return Err(WorkflowError::invalid_graph(format!(
                "workflow must have exactly one Start, found {}",
                start_ids.len()
            )));
        }
    }

    let start_id = match start_ids.into_iter().next() {
        Some(id) => id,
        None => return Err(WorkflowError::invalid_graph("workflow has no Start node")),
    };

    let mut end_ids: Vec<NodeId> = nodes
        .iter()
        .filter_map(|(id, spec)| match spec.node_type {
            NodeType::End => Some(id.clone()),
            _ => None,
        })
        .collect();
    end_ids.sort();

    match end_ids.is_empty() {
        true => return Err(WorkflowError::invalid_graph("workflow has no End node")),
        false => {}
    }

    detect_cycle(&nodes, &successors)?;

    for (id, spec) in &nodes {
        let outs = successors.get(id).map(|v| v.len()).unwrap_or(0);
        let ins = predecessors.get(id).map(|v| v.len()).unwrap_or(0);
        match (ins, outs, spec.node_type, id == &start_id) {
            (0, 0, _, true) => {}
            (0, 0, _, false) => {
                return Err(WorkflowError::invalid_graph(format!(
                    "isolated node: {}",
                    id
                )));
            }
            _ => {}
        }
    }

    Ok(GraphParts {
        nodes,
        successors,
        predecessors,
        branch_successors,
        start_id,
        end_ids,
    })
}

fn detect_cycle(
    nodes: &HashMap<NodeId, crate::definition::NodeSpec>,
    successors: &HashMap<NodeId, Vec<NodeId>>,
) -> Result<(), WorkflowError> {
    let mut color: HashMap<NodeId, VisitColor> = nodes
        .keys()
        .map(|id| (id.clone(), VisitColor::White))
        .collect();

    for start in nodes.keys() {
        match dfs_cycle(start, successors, &mut color) {
            Ok(()) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum VisitColor {
    White,
    Gray,
    Black,
}

fn dfs_cycle(
    node: &NodeId,
    successors: &HashMap<NodeId, Vec<NodeId>>,
    color: &mut HashMap<NodeId, VisitColor>,
) -> Result<(), WorkflowError> {
    match color.get(node).copied() {
        Some(VisitColor::Gray) => {
            return Err(WorkflowError::invalid_graph(format!(
                "cycle detected at node {}",
                node
            )));
        }
        Some(VisitColor::Black) => return Ok(()),
        Some(VisitColor::White) | None => {}
    }
    color.insert(node.clone(), VisitColor::Gray);

    let empty = Vec::new();
    let nexts = successors.get(node).unwrap_or(&empty);
    for next in nexts {
        dfs_cycle(next, successors, color)?;
    }

    color.insert(node.clone(), VisitColor::Black);
    Ok(())
}

/// 供测试：导出环检测可达性。
#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn has_duplicate_ids_for_test(definition: &WorkflowDefinition) -> bool {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    for node in &definition.nodes {
        if !seen.insert(node.id.clone()) {
            return true;
        }
    }
    false
}
