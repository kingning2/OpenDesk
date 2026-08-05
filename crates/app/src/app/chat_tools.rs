//! 进程内 MCP client 桥：把 `opendesk-mcp` 的只读数据查询工具暴露给聊天 LLM。
//!
//! 用两根交叉的 `tokio::io::duplex` 构造进程内 MCP 会话：server 端在后台 task 中
//! 运行 `rmcp::serve_server(OpendeskMcp)`，client 端经 `rmcp::serve_client` 握手后
//! 缓存工具定义、转发 `call_tool`。

use std::path::PathBuf;

use async_trait::async_trait;
use chat::{ChatTool, ChatToolCaller};
use opendesk_mcp::OpendeskMcp;
use rmcp::model::{CallToolRequestParams, RawContent};
use rmcp::{serve_client, serve_server, RoleClient};
use serde_json::Value;
use tokio::io::duplex;

/// duplex 管道容量（字节）。
const DUPLEX_CAPACITY: usize = 1 << 20;

/// 进程内只读数据库 MCP 桥。
pub struct ChatToolsBridge {
    client: rmcp::service::RunningService<RoleClient, ()>,
    _server_task: tokio::task::JoinHandle<()>,
    tools: Vec<ChatTool>,
}

impl ChatToolsBridge {
    /// 建立进程内 MCP 会话并缓存工具定义。
    ///
    /// # 参数
    /// - `data_dir` — OpenDesk 数据目录（`opendesk.db` / `crawler.db` 所在目录）
    ///
    /// # 返回值
    /// 握手或列出工具失败时返回错误描述。
    pub async fn new(data_dir: PathBuf) -> Result<Self, String> {
        // `tokio::io::duplex` 返回一对相连的流（服务端 / 客户端两端）。serve_server
        // 在握手后返回 RunningService，`waiting()` 使其持续服务直到连接关闭。
        let (server_transport, client_transport) = duplex(DUPLEX_CAPACITY);
        let server_task = tokio::spawn(async move {
            let server = OpendeskMcp::new(data_dir);
            let Ok(service) = serve_server(server, server_transport).await else {
                return;
            };
            let _ = service.waiting().await;
        });

        let client = serve_client((), client_transport)
            .await
            .map_err(|error| error.to_string())?;
        let tools = client
            .peer()
            .list_all_tools()
            .await
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|tool| ChatTool {
                name: tool.name.to_string(),
                description: tool
                    .description
                    .map(|desc| desc.to_string())
                    .unwrap_or_default(),
                parameters: Value::Object((*tool.input_schema).clone()),
            })
            .collect();

        Ok(Self {
            client,
            _server_task: server_task,
            tools,
        })
    }
}

#[async_trait]
impl ChatToolCaller for ChatToolsBridge {
    fn list_tools(&self) -> Vec<ChatTool> {
        self.tools.clone()
    }

    async fn call_tool(&self, name: &str, args: &Value) -> Result<Value, String> {
        let arguments = match args {
            Value::Object(map) => Some(map.clone()),
            _ => None,
        };
        let result = self
            .client
            .peer()
            .call_tool(CallToolRequestParams {
                meta: None,
                name: name.to_string().into(),
                arguments,
                task: None,
            })
            .await
            .map_err(|error| error.to_string())?;

        let text = result
            .content
            .iter()
            .filter_map(|content| match &content.raw {
                RawContent::Text(raw) => Some(raw.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");

        if result.is_error == Some(true) {
            return Err(if text.is_empty() {
                "tool call failed".into()
            } else {
                text
            });
        }
        Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)))
    }
}
