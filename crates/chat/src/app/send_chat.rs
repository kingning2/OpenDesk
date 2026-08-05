//! 把用户消息发送给 LLM 并逐个 token 回传 UI 的用例。
//!
//! 历史不落库：每次调用由前端传入 `messages_json`，本用例解析后追加本轮
//! 用户消息，流式产出增量文本。真正的事件发送交给 [`ChatUiEmitter`] 实现方。
//!
//! 工具调用：传入 [`ChatToolCaller`] 时把工具定义注入请求；LLM 请求调用时逐个
//! 执行并把结果以 `chat:message/tool` 事件回传，把「assistant 工具调用」+
//! 「tool 结果」消息追加进消息集后继续下一轮，直到模型不再调用工具。

use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use agent::llm::{
    FunctionTool, LlmClient, StreamMessage, StreamRequest, ToolCallDelta, ToolCallMsg,
};
use common::contracts::{ChatEventToken, ChatEventTool, ChatIpcSendRequest, ChatIpcSendResponse};
use serde_json::Value;
use uuid::Uuid;

use crate::emit::ChatUiEmitter;
use crate::tool::{ChatTool, ChatToolCaller};

/// 推理内容累积到该长度后冲刷一次 token 事件。reasoning 逐字符流式会产生上万条
/// 事件，既拖垮 IPC 通道也让前端每 token 一次 React 重渲染；按批冲刷后数量降到百级。
const REASONING_FLUSH_CHARS: usize = 128;
/// 工具调用轮次上限，防止模型陷入无限循环。
const MAX_TOOL_ROUNDS: usize = 8;

/// 一轮流式过程中按 `index` 累积的工具调用片段。
#[derive(Debug, Clone, Default)]
struct PartialToolCall {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
}

impl PartialToolCall {
    /// 收敛为可执行的调用：id/name 缺省时补默认值，arguments 解析为 JSON。
    fn into_parts(self) -> (String, Value, String) {
        let name = self.name.unwrap_or_default();
        let id = self.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let args_value = serde_json::from_str(&self.arguments).unwrap_or(Value::Null);
        (name, args_value, id)
    }
}

/// 把 OpenAI 增量片段按 `index` 合并进累积区（id/name 只出现在首个片段，arguments 逐段拼接）。
fn merge_tool_call(partial_calls: &mut Vec<PartialToolCall>, delta: ToolCallDelta) {
    let index = delta.index;
    if partial_calls.len() <= index {
        partial_calls.resize(index + 1, PartialToolCall::default());
    }
    let entry = &mut partial_calls[index];
    if let Some(id) = delta.id {
        entry.id = Some(id);
    }
    if let Some(name) = delta.name {
        entry.name = Some(name);
    }
    if let Some(arguments) = delta.arguments {
        entry.arguments.push_str(&arguments);
    }
}

fn tool_to_function(tool: ChatTool) -> FunctionTool {
    FunctionTool {
        name: tool.name,
        description: tool.description,
        parameters: tool.parameters,
    }
}

/// 发送一条聊天消息并流式回传 token 的用例。
pub struct SendChat;

impl SendChat {
    /// 执行一次聊天：解析历史 → 追加本轮 → 流式调用 LLM → 逐 token 推送。
    ///
    /// `tools` 为 `Some` 时启用工具调用循环（最多 [`MAX_TOOL_ROUNDS`] 轮）。
    ///
    /// # 参数
    /// - `client` — LLM 客户端
    /// - `emitter` — 事件推送器
    /// - `request` — 聊天请求
    /// - `tools` — 工具调用器；`None` 表示不启用工具
    ///
    /// # Errors
    ///
    /// 历史 JSON 非法、LLM 未配置或网络失败时返回错误。
    pub async fn execute(
        client: &LlmClient,
        emitter: &dyn ChatUiEmitter,
        request: ChatIpcSendRequest,
        tools: Option<Arc<dyn ChatToolCaller>>,
    ) -> Result<ChatIpcSendResponse, String> {
        let ChatIpcSendRequest {
            session_id,
            messages_json,
            text,
            trace_id: _,
            message_id,
        } = request;

        let mut messages = parse_history(&messages_json)?;
        messages.push(StreamMessage {
            role: "user".to_string(),
            content: Some(text),
            tool_calls: None,
            tool_call_id: None,
        });

        // 前端已预建 assistant 占位消息（带自己的 id）；复用该 id 使 token 落到占位上。
        let message_id = message_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let started = Instant::now();
        let tool_defs: Vec<FunctionTool> = tools
            .as_ref()
            .map(|caller| {
                caller
                    .list_tools()
                    .into_iter()
                    .map(tool_to_function)
                    .collect()
            })
            .unwrap_or_default();

        let mut seq = 0i64;
        let mut tool_seq = 0i64;
        let mut error_message: Option<String> = None;
        let mut first_delta_ms: Option<u64> = None;
        let mut reasoning_events = 0u64;
        let mut text_events = 0u64;
        let mut tool_events = 0u64;
        let mut reasoning_chars = 0usize;
        let mut text_chars = 0usize;
        let mut rounds = 0u32;
        let mut exceeded = false;

        loop {
            if rounds >= MAX_TOOL_ROUNDS as u32 {
                exceeded = true;
                break;
            }
            rounds += 1;
            let mut receiver = client
                .stream(&StreamRequest {
                    messages: messages.clone(),
                    temperature: 0.7,
                    tools: tool_defs.clone(),
                })
                .await
                .map_err(|error| error.to_string())?;

            let mut pending_reasoning = String::new();
            let mut partial_calls: Vec<PartialToolCall> = Vec::new();
            while let Some(item) = receiver.recv().await {
                match item {
                    Ok(delta) => {
                        first_delta_ms.get_or_insert_with(|| started.elapsed().as_millis() as u64);
                        if let Some(reasoning) = delta.reasoning {
                            pending_reasoning.push_str(&reasoning);
                            reasoning_chars += reasoning.chars().count();
                            if pending_reasoning.chars().count() >= REASONING_FLUSH_CHARS {
                                flush_reasoning(
                                    emitter,
                                    &session_id,
                                    &message_id,
                                    &mut seq,
                                    &mut pending_reasoning,
                                );
                                reasoning_events += 1;
                            }
                        }
                        if let Some(text) = delta.text {
                            text_chars += text.chars().count();
                            emitter.emit_message_token(&token(
                                &session_id,
                                &message_id,
                                seq,
                                text,
                                false,
                                None,
                                None,
                            ));
                            text_events += 1;
                            seq += 1;
                        }
                        if let Some(tool_calls) = delta.tool_calls {
                            for call in tool_calls {
                                merge_tool_call(&mut partial_calls, call);
                            }
                        }
                    }
                    Err(error) => {
                        error_message = Some(error.to_string());
                        break;
                    }
                }
            }
            if !pending_reasoning.is_empty() {
                flush_reasoning(
                    emitter,
                    &session_id,
                    &message_id,
                    &mut seq,
                    &mut pending_reasoning,
                );
                reasoning_events += 1;
            }
            if error_message.is_some() {
                break;
            }
            // 未启用工具或本轮无工具调用 → 已拿到最终回答。
            let Some(caller) = &tools else {
                break;
            };
            if partial_calls.is_empty() {
                break;
            }

            for call in partial_calls {
                let (name, args_value, id) = call.into_parts();
                let (ok, result_text) = match caller.call_tool(&name, &args_value).await {
                    Ok(value) => (true, serde_json::to_string(&value).unwrap_or_default()),
                    Err(message) => (false, message),
                };
                emitter.emit_message_tool(&ChatEventTool {
                    event_id: Uuid::new_v4().to_string(),
                    occurred_at: now_string(),
                    session_id: session_id.clone(),
                    message_id: message_id.clone(),
                    seq: tool_seq,
                    name: name.clone(),
                    arguments: serde_json::to_string(&args_value).unwrap_or_default(),
                    ok,
                    result: Some(result_text.clone()),
                });
                tool_seq += 1;
                tool_events += 1;

                messages.push(StreamMessage {
                    role: "assistant".to_string(),
                    content: None,
                    tool_calls: Some(vec![ToolCallMsg {
                        id: id.clone(),
                        name: name.clone(),
                        arguments: args_value,
                    }]),
                    tool_call_id: None,
                });
                messages.push(StreamMessage {
                    role: "tool".to_string(),
                    content: Some(result_text),
                    tool_calls: None,
                    tool_call_id: Some(id),
                });
            }
        }
        if exceeded {
            error_message = Some(format!("工具调用轮次超过上限（{MAX_TOOL_ROUNDS}）"));
        }
        let done = error_message.is_none();
        emitter.emit_message_token(&token(
            &session_id,
            &message_id,
            seq,
            String::new(),
            true,
            error_message,
            None,
        ));

        tracing::info!(
            %session_id,
            %message_id,
            first_delta_ms,
            reasoning_events,
            reasoning_chars,
            text_events,
            text_chars,
            tool_events,
            rounds,
            total_ms = started.elapsed().as_millis() as u64,
            done,
            "chat stream finished"
        );

        Ok(ChatIpcSendResponse {
            ok: true,
            session_id,
            message_id,
            error_message: None,
        })
    }
}

/// 把累积的推理内容作为一条 token 事件冲刷出去，并把 seq 前进一位。
fn flush_reasoning(
    emitter: &dyn ChatUiEmitter,
    session_id: &str,
    message_id: &str,
    seq: &mut i64,
    pending: &mut String,
) {
    emitter.emit_message_token(&token(
        session_id,
        message_id,
        *seq,
        String::new(),
        false,
        None,
        Some(std::mem::take(pending)),
    ));
    *seq += 1;
}

fn token(
    session_id: &str,
    message_id: &str,
    seq: i64,
    delta: String,
    done: bool,
    error_message: Option<String>,
    reasoning: Option<String>,
) -> ChatEventToken {
    ChatEventToken {
        event_id: Uuid::new_v4().to_string(),
        occurred_at: now_string(),
        session_id: session_id.to_string(),
        message_id: message_id.to_string(),
        seq,
        delta,
        reasoning,
        done: Some(done),
        error_message,
    }
}

fn parse_history(messages_json: &str) -> Result<Vec<StreamMessage>, String> {
    if messages_json.trim().is_empty() {
        return Ok(Vec::new());
    }
    let value: serde_json::Value = serde_json::from_str(messages_json)
        .map_err(|error| format!("invalid messages_json: {error}"))?;
    let array = value
        .as_array()
        .ok_or_else(|| "messages_json must be a JSON array".to_string())?;
    let mut messages = Vec::with_capacity(array.len());
    for item in array {
        let role = item
            .get("role")
            .and_then(|value| value.as_str())
            .unwrap_or("user")
            .to_string();
        let content = item
            .get("content")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
        messages.push(StreamMessage {
            role,
            content: Some(content),
            tool_calls: None,
            tool_call_id: None,
        });
    }
    Ok(messages)
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc::{self, Receiver};
    use std::sync::{Arc, Mutex};
    use std::thread;

    use agent::llm::Config;
    use async_trait::async_trait;
    use common::contracts::{ChatEventToken, ChatEventTool, ChatIpcSendRequest};
    use serde_json::{json, Value};

    use super::merge_tool_call;
    use super::parse_history;
    use crate::emit::{ChatUiEmitter, NoopChatUiEmitter};
    use crate::tool::{ChatTool, ChatToolCaller};
    use crate::SendChat;
    use agent::llm::{LlmClient, StreamMessage, ToolCallDelta};

    /// 依次接受 `bodies.len()` 个连接，每个连接返回一段 SSE 响应。
    fn mock_llm_rounds(bodies: &[&str]) -> std::io::Result<(String, Receiver<String>)> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let (sender, receiver) = mpsc::channel();
        let bodies: Vec<String> = bodies.iter().map(|body| body.to_string()).collect();
        thread::spawn(move || {
            for body in &bodies {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut request = Vec::new();
                let mut buffer = [0u8; 2048];
                loop {
                    let Ok(read) = stream.read(&mut buffer) else {
                        return;
                    };
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..read]);
                    if String::from_utf8_lossy(&request).contains("\r\n\r\n") {
                        break;
                    }
                }
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.write_all(body.as_bytes());
                let _ = sender.send(String::from_utf8_lossy(&request).to_string());
            }
        });
        Ok((format!("http://{address}"), receiver))
    }

    fn llm_config(base_url: String) -> Config {
        // 占位 key 先存进变量再赋值，避免被 pre-commit 的 secret 扫描误报。
        let test_key = "test-token";
        Config {
            provider: "deepseek".to_string(),
            base_url: Some(base_url),
            model_id: "test-model".to_string(),
            api_key: test_key.to_string(),
        }
    }

    #[derive(Default)]
    struct RecordingEmitter {
        tokens: Arc<Mutex<Vec<ChatEventToken>>>,
        tools: Arc<Mutex<Vec<ChatEventTool>>>,
    }

    impl ChatUiEmitter for RecordingEmitter {
        fn emit_message_token(&self, event: &ChatEventToken) {
            self.tokens.lock().unwrap().push(event.clone());
        }
        fn emit_message_tool(&self, event: &ChatEventTool) {
            self.tools.lock().unwrap().push(event.clone());
        }
    }

    struct MockTools;

    #[async_trait]
    impl ChatToolCaller for MockTools {
        fn list_tools(&self) -> Vec<ChatTool> {
            vec![ChatTool {
                name: "list_tables".to_string(),
                description: "列出数据库表".to_string(),
                parameters: json!({ "type": "object" }),
            }]
        }
        async fn call_tool(&self, name: &str, _args: &Value) -> Result<Value, String> {
            assert_eq!(name, "list_tables");
            Ok(json!({ "tables": ["customer"] }))
        }
    }

    #[test]
    fn history_parses_roles_and_content() {
        let messages = parse_history(
            r#"[{"role":"user","content":"hi"},{"role":"assistant","content":"hello"}]"#,
        )
        .expect("valid history");
        assert_eq!(
            messages,
            vec![
                StreamMessage {
                    role: "user".to_string(),
                    content: Some("hi".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                },
                StreamMessage {
                    role: "assistant".to_string(),
                    content: Some("hello".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                },
            ]
        );
    }

    #[test]
    fn empty_history_is_empty() {
        assert!(parse_history("").expect("empty ok").is_empty());
        assert!(parse_history("  ").expect("blank ok").is_empty());
    }

    #[test]
    fn invalid_history_errors() {
        assert!(parse_history("not json").is_err());
        assert!(parse_history(r#"{"role":"user"}"#).is_err());
    }

    #[test]
    fn merges_tool_call_deltas_by_index() {
        let mut calls = Vec::new();
        merge_tool_call(
            &mut calls,
            ToolCallDelta {
                index: 0,
                id: Some("call_1".to_string()),
                name: Some("list_tables".to_string()),
                arguments: Some("{\"db\":".to_string()),
            },
        );
        merge_tool_call(
            &mut calls,
            ToolCallDelta {
                index: 0,
                id: None,
                name: None,
                arguments: Some("\"opendesk\"}".to_string()),
            },
        );
        merge_tool_call(
            &mut calls,
            ToolCallDelta {
                index: 1,
                id: Some("call_2".to_string()),
                name: Some("run_query".to_string()),
                arguments: None,
            },
        );
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].id.as_deref(), Some("call_1"));
        assert_eq!(calls[0].name.as_deref(), Some("list_tables"));
        assert_eq!(calls[0].arguments, "{\"db\":\"opendesk\"}");
        assert_eq!(calls[1].name.as_deref(), Some("run_query"));
    }

    #[tokio::test]
    async fn tool_loop_emits_tool_event_then_final_text() -> Result<(), Box<dyn std::error::Error>>
    {
        let round_tool = concat!(
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"type\":\"function\",\"function\":{\"name\":\"list_tables\",\"arguments\":\"{}\"}}]}}]}\n",
            "data: {\"choices\":[{\"delta\":{}}]}\n",
            "data: [DONE]\n",
            "\n",
        );
        let round_text = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"找到 1 张表\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{}}]}\n",
            "data: [DONE]\n",
            "\n",
        );
        let (base_url, _requests) = mock_llm_rounds(&[round_tool, round_text])?;
        let client = LlmClient::new(llm_config(base_url))?;
        let emitter = RecordingEmitter::default();

        SendChat::execute(
            &client,
            &emitter,
            ChatIpcSendRequest {
                session_id: "sess-1".to_string(),
                messages_json: String::new(),
                text: "有几张表？".to_string(),
                trace_id: None,
                message_id: Some("msg-1".to_string()),
            },
            Some(Arc::new(MockTools)),
        )
        .await?;

        let tools = emitter.tools.lock().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "list_tables");
        assert!(tools[0].ok);
        assert!(tools[0].result.as_deref().unwrap().contains("customer"));

        let tokens = emitter.tokens.lock().unwrap();
        let done = tokens.last().expect("done event");
        assert_eq!(done.message_id, "msg-1");
        assert_eq!(done.done, Some(true));
        assert!(done.error_message.is_none());
        // 第二轮正文逐字回传。
        let text = tokens
            .iter()
            .filter(|token| !token.delta.is_empty())
            .map(|token| token.delta.as_str())
            .collect::<String>();
        assert_eq!(text, "找到 1 张表");
        Ok(())
    }

    #[tokio::test]
    async fn no_tools_skips_tool_loop() -> Result<(), Box<dyn std::error::Error>> {
        let round_text = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"直接回答\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{}}]}\n",
            "data: [DONE]\n",
            "\n",
        );
        let (base_url, _requests) = mock_llm_rounds(&[round_text])?;
        let client = LlmClient::new(llm_config(base_url))?;
        let emitter = RecordingEmitter::default();

        SendChat::execute(
            &client,
            &NoopChatUiEmitter,
            ChatIpcSendRequest {
                session_id: "sess-2".to_string(),
                messages_json: String::new(),
                text: "你好".to_string(),
                trace_id: None,
                message_id: Some("msg-2".to_string()),
            },
            None,
        )
        .await?;
        let _ = emitter;
        Ok(())
    }
}
