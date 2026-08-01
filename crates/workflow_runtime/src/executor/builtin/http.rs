//! Http 最小适配执行器（不发起真实网络；写入 Context 占位）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use crate::context::ContextPatch;
use crate::definition::NodeType;
use crate::error::WorkflowError;
use crate::executor::traits::{ExecuteInput, ExecuteOutput, NodeExecutor};
use async_trait::async_trait;
use serde_json::{json, Value};

/// Http 占位：记录 url/method，返回 stub 状态。
///
/// @author Xiaoman
/// @created 2026-07-23
pub struct HttpExecutor;

#[async_trait]
impl NodeExecutor for HttpExecutor {
    fn node_type(&self) -> NodeType {
        NodeType::Http
    }

    async fn execute(&self, input: ExecuteInput<'_>) -> Result<ExecuteOutput, WorkflowError> {
        let url = match input.config.get("url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => {
                return Err(WorkflowError::node_execution(
                    "Http node requires config.url",
                ));
            }
        };
        let method = match input.config.get("method").and_then(|v| v.as_str()) {
            Some(method) => method,
            None => "GET",
        };

        Ok(ExecuteOutput {
            context_patches: vec![ContextPatch {
                path: format!("nodes.{}.http", input.node_id),
                value: json!({
                    "url": url,
                    "method": method,
                    "status": "stub_ok",
                }),
            }],
            message: Some(format!("http stub {method} {url}")),
            branch: None,
        })
    }
}

/// 避免未使用 Value 警告（保留扩展点）。
#[allow(dead_code)]
fn _unused() -> Value {
    Value::Null
}
