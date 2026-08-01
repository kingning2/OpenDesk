//! End 节点。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use crate::definition::NodeType;
use crate::error::WorkflowError;
use crate::executor::traits::{ExecuteInput, ExecuteOutput, NodeExecutor};
use async_trait::async_trait;

/// End 执行器（空操作）。
///
/// @author Xiaoman
/// @created 2026-07-23
pub struct EndExecutor;

#[async_trait]
impl NodeExecutor for EndExecutor {
    fn node_type(&self) -> NodeType {
        NodeType::End
    }

    async fn execute(&self, _input: ExecuteInput<'_>) -> Result<ExecuteOutput, WorkflowError> {
        Ok(ExecuteOutput {
            message: Some("end".to_string()),
            ..ExecuteOutput::default()
        })
    }
}
