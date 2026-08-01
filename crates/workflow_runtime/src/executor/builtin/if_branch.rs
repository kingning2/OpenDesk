//! If 分支节点。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use crate::context::ContextPatch;
use crate::definition::NodeType;
use crate::error::WorkflowError;
use crate::executor::traits::{ExecuteInput, ExecuteOutput, NodeExecutor};
use async_trait::async_trait;
use serde_json::Value;

/// If 执行器：读取 context 路径布尔，输出 branch `true`/`false`。
///
/// config: `{ "path": "flags.enabled" }` 或 `{ "literal": true }`
///
/// @author Xiaoman
/// @created 2026-07-23
pub struct IfExecutor;

#[async_trait]
impl NodeExecutor for IfExecutor {
    fn node_type(&self) -> NodeType {
        NodeType::If
    }

    async fn execute(&self, input: ExecuteInput<'_>) -> Result<ExecuteOutput, WorkflowError> {
        let value = match input.config.get("literal") {
            Some(literal) => literal.clone(),
            None => {
                let path = match input.config.get("path").and_then(|v| v.as_str()) {
                    Some(path) => path,
                    None => {
                        return Err(WorkflowError::node_execution(
                            "If node requires config.path or config.literal",
                        ));
                    }
                };
                input.context.get_path(path)?.clone()
            }
        };

        let is_true = match value {
            Value::Bool(flag) => flag,
            Value::Null => false,
            Value::Number(num) => match num.as_i64() {
                Some(0) => false,
                Some(_) => true,
                None => true,
            },
            Value::String(text) => !text.is_empty() && text != "false" && text != "0",
            Value::Array(items) => !items.is_empty(),
            Value::Object(map) => !map.is_empty(),
        };

        let branch = match is_true {
            true => "true",
            false => "false",
        };

        Ok(ExecuteOutput {
            branch: Some(branch.to_string()),
            context_patches: vec![ContextPatch {
                path: format!("nodes.{}.branch", input.node_id),
                value: Value::String(branch.to_string()),
            }],
            message: Some(format!("if => {branch}")),
        })
    }
}
