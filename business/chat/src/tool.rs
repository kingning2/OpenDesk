//! 聊天过程中的工具调用抽象。
//!
//! `SendChat` 通过 [`ChatToolCaller`] 执行 LLM 请求的工具；进程内 MCP client 桥
//! 在 `crates/app` 实现该 trait。`list_tools` 同步返回定义（供请求携带 `tools`），
//! `call_tool` 异步执行并返回 JSON 值。

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

/// 一个可调用工具的定义。
#[derive(Debug, Clone)]
pub struct ChatTool {
    /// 工具名（与 MCP 工具名一致）。
    pub name: String,
    /// 工具说明（供 LLM 选择工具）。
    pub description: String,
    /// 参数 JSON Schema 对象。
    pub parameters: Value,
}

/// 聊天工具调用器。
///
/// 作者：coisini
#[async_trait]
pub trait ChatToolCaller: Send + Sync {
    /// 返回可注入请求的工具定义。
    fn list_tools(&self) -> Vec<ChatTool>;

    /// 执行一次工具调用。
    ///
    /// # 参数
    /// - `name` — 工具名
    /// - `args` — 工具参数（JSON 对象）
    ///
    /// # 返回值
    /// 成功时为工具结果 JSON 值；失败时返回错误描述。
    async fn call_tool(&self, name: &str, args: &Value) -> Result<Value, String>;
}

/// 组合多个工具调用器：`list_tools` 拼接各 caller 的工具，`call_tool` 按工具名分发。
#[derive(Default)]
pub struct CompositeChatToolCaller {
    callers: Vec<Arc<dyn ChatToolCaller>>,
}

impl CompositeChatToolCaller {
    /// 组装组合调用器。
    pub fn new(callers: Vec<Arc<dyn ChatToolCaller>>) -> Self {
        Self { callers }
    }

    fn owner_of(&self, name: &str) -> Option<Arc<dyn ChatToolCaller>> {
        self.callers
            .iter()
            .find(|caller| caller.list_tools().iter().any(|tool| tool.name == name))
            .cloned()
    }
}

#[async_trait]
impl ChatToolCaller for CompositeChatToolCaller {
    fn list_tools(&self) -> Vec<ChatTool> {
        self.callers
            .iter()
            .flat_map(|caller| caller.list_tools())
            .collect()
    }

    async fn call_tool(&self, name: &str, args: &Value) -> Result<Value, String> {
        let owner = self
            .owner_of(name)
            .ok_or_else(|| format!("unknown tool: {name}"))?;
        owner.call_tool(name, args).await
    }
}
