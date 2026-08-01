//! Switch 分支节点。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use crate::context::ContextPatch;
use crate::definition::NodeType;
use crate::error::WorkflowError;
use crate::executor::traits::{ExecuteInput, ExecuteOutput, NodeExecutor};
use async_trait::async_trait;
use serde_json::Value;

/// Switch：从 context 路径取字符串作为 branch；缺省 `default`。
///
/// config: `{ "path": "route.key" }`
///
/// @author Xiaoman
/// @created 2026-07-23
pub struct SwitchExecutor;

#[async_trait]
impl NodeExecutor for SwitchExecutor {
    fn node_type(&self) -> NodeType {
        NodeType::Switch
    }

    async fn execute(&self, input: ExecuteInput<'_>) -> Result<ExecuteOutput, WorkflowError> {
        let path = match input.config.get("path").and_then(|v| v.as_str()) {
            Some(path) => path,
            None => {
                return Err(WorkflowError::node_execution(
                    "Switch node requires config.path",
                ));
            }
        };

        let branch = match input.context.get_path(path) {
            Ok(Value::String(text)) => text.clone(),
            Ok(other) => other.to_string(),
            Err(_) => "default".to_string(),
        };

        Ok(ExecuteOutput {
            branch: Some(branch.clone()),
            context_patches: vec![ContextPatch {
                path: format!("nodes.{}.branch", input.node_id),
                value: Value::String(branch.clone()),
            }],
            message: Some(format!("switch => {branch}")),
        })
    }
}
