//! 工作流定义与运行策略。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use super::{EdgeSpec, NodeSpec};
use crate::id::{NodeId, WorkflowId};
use serde::{Deserialize, Serialize};

/// 实例级运行策略（不在 DAG 内造环）。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RunPolicy {
    /// 跑完即止。
    #[default]
    Once,
    /// 成功结束后从指定节点重启（如采集 auto_loop）。
    OnSuccessRestartFrom {
        /// 重启入口节点。
        node_id: NodeId,
    },
}

/// 静态工作流定义。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// 定义 id。
    pub id: WorkflowId,
    /// 节点列表。
    pub nodes: Vec<NodeSpec>,
    /// 边列表。
    pub edges: Vec<EdgeSpec>,
    /// 运行策略。
    #[serde(default)]
    pub run_policy: RunPolicy,
}
