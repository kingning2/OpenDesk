//! 边定义。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use crate::id::NodeId;
use serde::{Deserialize, Serialize};

/// 有向边；可选分支键（If/Switch）。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeSpec {
    /// 边 id。
    pub id: String,
    /// 源节点。
    pub source: NodeId,
    /// 目标节点。
    pub target: NodeId,
    /// 分支键；默认边为空。
    pub branch: Option<String>,
}
