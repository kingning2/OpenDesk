//! AI 最小适配执行器（不调用 LLM；写入 Context 占位）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use crate::context::ContextPatch;
use crate::definition::NodeType;
use crate::error::WorkflowError;
use crate::executor::traits::{ExecuteInput, ExecuteOutput, NodeExecutor};
use async_trait::async_trait;
use serde_json::json;

/// AI 占位执行器。
///
/// @author Xiaoman
/// @created 2026-07-23
pub struct AiExecutor;

#[async_trait]
impl NodeExecutor for AiExecutor {
    fn node_type(&self) -> NodeType {
        NodeType::Ai
    }

    async fn execute(&self, input: ExecuteInput<'_>) -> Result<ExecuteOutput, WorkflowError> {
        let prompt = match input.config.get("prompt").and_then(|v| v.as_str()) {
            Some(prompt) => prompt,
            None => "",
        };

        Ok(ExecuteOutput {
            context_patches: vec![ContextPatch {
                path: format!("nodes.{}.ai", input.node_id),
                value: json!({
                    "prompt": prompt,
                    "output": "stub_ai_output",
                }),
            }],
            message: Some("ai stub completed".to_string()),
            branch: None,
        })
    }
}
