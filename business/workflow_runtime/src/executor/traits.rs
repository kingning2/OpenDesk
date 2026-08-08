//! NodeExecutor 契约。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use crate::context::{ContextPatch, WorkflowContext};
use crate::definition::NodeType;
use crate::error::WorkflowError;
use crate::id::NodeId;
use async_trait::async_trait;
use serde_json::Value;

/// 执行输入。
///
/// @author coisini
/// @created 2026-07-23
pub struct ExecuteInput<'a> {
    /// 节点 id。
    pub node_id: &'a NodeId,
    /// 节点配置。
    pub config: &'a Value,
    /// 只读 Context 快照。
    pub context: &'a WorkflowContext,
    /// 当前 attempt（从 1 起）。
    pub attempt: u32,
}

/// 执行输出。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Debug, Clone, Default)]
pub struct ExecuteOutput {
    /// Context 补丁。
    pub context_patches: Vec<ContextPatch>,
    /// If/Switch 分支键。
    pub branch: Option<String>,
    /// 可选消息。
    pub message: Option<String>,
}

/// 节点执行器统一 Trait。
///
/// @author coisini
/// @created 2026-07-23
#[async_trait]
pub trait NodeExecutor: Send + Sync {
    /// 本执行器对应的节点类型。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns 节点类型
    fn node_type(&self) -> NodeType;

    /// 执行节点；禁止自行 Retry / 更新 UI。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @param input - 执行输入
    /// @returns 输出或错误
    async fn execute(&self, input: ExecuteInput<'_>) -> Result<ExecuteOutput, WorkflowError>;
}
