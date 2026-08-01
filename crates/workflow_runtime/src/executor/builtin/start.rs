//! Start 节点。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use crate::definition::NodeType;
use crate::error::WorkflowError;
use crate::executor::traits::{ExecuteInput, ExecuteOutput, NodeExecutor};
use async_trait::async_trait;

/// Start 执行器（空操作）。
///
/// @author Xiaoman
/// @created 2026-07-23
pub struct StartExecutor;

#[async_trait]
impl NodeExecutor for StartExecutor {
    fn node_type(&self) -> NodeType {
        NodeType::Start
    }

    async fn execute(&self, _input: ExecuteInput<'_>) -> Result<ExecuteOutput, WorkflowError> {
        Ok(ExecuteOutput {
            message: Some("start".to_string()),
            ..ExecuteOutput::default()
        })
    }
}
