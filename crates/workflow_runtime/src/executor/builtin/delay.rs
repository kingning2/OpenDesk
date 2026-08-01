//! Delay 节点。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use crate::definition::NodeType;
use crate::error::WorkflowError;
use crate::executor::traits::{ExecuteInput, ExecuteOutput, NodeExecutor};
use async_trait::async_trait;
use std::time::Duration;

/// 延迟执行器；config.delay_ms。
///
/// @author Xiaoman
/// @created 2026-07-23
pub struct DelayExecutor;

#[async_trait]
impl NodeExecutor for DelayExecutor {
    fn node_type(&self) -> NodeType {
        NodeType::Delay
    }

    async fn execute(&self, input: ExecuteInput<'_>) -> Result<ExecuteOutput, WorkflowError> {
        let delay_ms = match input.config.get("delay_ms").and_then(|v| v.as_u64()) {
            Some(value) => value,
            None => 0,
        };
        match delay_ms > 0 {
            true => tokio::time::sleep(Duration::from_millis(delay_ms)).await,
            false => {}
        }
        Ok(ExecuteOutput {
            message: Some(format!("delayed {delay_ms}ms")),
            ..ExecuteOutput::default()
        })
    }
}
