//! 把用户消息发送给 LLM 并逐个 token 回传 UI 的用例。
//!
//! 历史不落库：每次调用由前端传入 `messages_json`，本用例解析后追加本轮
//! 用户消息，流式产出增量文本。真正的事件发送交给 [`ChatUIEmitter`] 实现方。
//!
//! 工具调用：传入 [`ChatToolCaller`] 时把工具定义注入请求；LLM 请求调用时逐个
//! 执行并把结果以 `chat:message/tool` 事件回传，把「assistant 工具调用」+
//! 「tool 结果」消息追加进消息集后继续下一轮，直到模型不再调用工具。

use std::sync::Arc;
use std::time::Instant;

use agent::embedding::Embedder;
use agent::llm::{
    FunctionTool, LlmClient, StreamMessage, StreamRequest, ToolCallDelta, ToolCallMsg,
};
use agent::skills::SkillRegistry;
use common::contracts::{ChatEventToken, ChatEventTool, ChatIpcSendRequest, ChatIpcSendResponse};
use common::tools::time::now_secs_string;
use ports::chat::{ChatMemoryStore, ChatStore, SaveChatMessage};
use ports::knowledge::KnowledgeStore;
use serde_json::{json, Value};
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::emit::ChatUIEmitter;
use crate::tool::{ChatTool, ChatToolCaller};

/// 推理内容累积到该长度后冲刷一次 token 事件。reasoning 逐字符流式会产生上万条
/// 事件，既拖垮 IPC 通道也让前端每 token 一次 React 重渲染；按批冲刷后数量降到百级。
const REASONING_FLUSH_CHARS: usize = 128;
/// 工具调用轮次上限，防止模型陷入无限循环。
const MAX_TOOL_ROUNDS: usize = 8;
/// 单会话历史窗口上限（条）：超过后最旧批次用会话 digest 摘要替换，控制 token 预算。
const WINDOW_MAX_MESSAGES: usize = 30;
/// 窗口压缩后保留的最近消息条数。
const WINDOW_KEEP_MESSAGES: usize = 20;
/// 每次发送从长期记忆检索注入的相关记忆条数。
const MEMORY_TOP_K: usize = 5;
/// 每次发送从知识库检索注入的相关文档分块条数。
const KNOWLEDGE_TOP_K: usize = 5;

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
    /// `store` 为 `Some` 时历史从 `chat.db` 重建、用户/助手消息自动落库（多会话
    /// 持久化模式）；为 `None` 时退化为解析前端上传的 `messages_json`（测试/未接线）。
    ///
    /// `memory` + `embedder` 同时为 `Some` 时启用长期记忆：检索相关记忆注入
    /// 上下文，历史超窗时用会话 digest 摘要压缩最旧批次（均为 best-effort）。
    ///
    /// `knowledge` 为 `Some` 时启用知识库检索：嵌入用户文本检索 top-k 文档分块
    /// 注入上下文（best-effort，失败不阻塞发送）。
    ///
    /// # 参数
    /// - `client` — LLM 客户端
    /// - `emitter` — 事件推送器
    /// - `request` — 聊天请求
    /// - `tools` — 工具调用器；`None` 表示不启用工具
    /// - `skills` — 系统操作指引 Skill 注册表；`Some` 时注入一条 system 指引消息
    /// - `store` — 会话/消息持久化端口；`None` 表示不落库
    /// - `memory` — 长期记忆端口；`None` 表示不启用记忆
    /// - `embedder` — 本地嵌入服务；`None` 表示不启用记忆检索
    /// - `knowledge` — 知识库检索端口；`None` 表示不启用知识库检索
    ///
    /// # Errors
    ///
    /// 历史 JSON 非法、LLM 未配置或网络失败时返回错误。
    #[allow(clippy::too_many_arguments)]
    pub async fn execute(
        client: &LlmClient,
        emitter: &dyn ChatUIEmitter,
        request: ChatIpcSendRequest,
        tools: Option<Arc<dyn ChatToolCaller>>,
        skills: Option<Arc<SkillRegistry>>,
        store: Option<&dyn ChatStore>,
        memory: Option<Arc<dyn ChatMemoryStore>>,
        embedder: Option<Arc<dyn Embedder>>,
        knowledge: Option<Arc<dyn KnowledgeStore>>,
    ) -> Result<ChatIpcSendResponse, String> {
        let ChatIpcSendRequest {
            session_id,
            messages_json,
            text,
            trace_id: _,
            message_id,
        } = request;

        let mut messages = match store {
            Some(store) => {
                let records = store
                    .load_messages(&session_id)
                    .map_err(|error| format!("加载会话历史失败: {error}"))?;
                records
                    .into_iter()
                    .map(|record| StreamMessage {
                        role: record.role,
                        content: Some(record.content),
                        tool_calls: None,
                        tool_call_id: None,
                    })
                    .collect()
            }
            None => parse_history(messages_json.as_deref().unwrap_or(""))?,
        };

        // 窗口压缩：历史超窗时用该会话最新 digest 摘要替换最旧批次，控制 token 预算。
        if let Some(memory) = &memory {
            if messages.len() > WINDOW_MAX_MESSAGES {
                if let Ok(Some(digest)) = memory.latest_session_digest(&session_id) {
                    let kept = messages.split_off(messages.len() - WINDOW_KEEP_MESSAGES);
                    let mut compacted = Vec::with_capacity(kept.len() + 1);
                    compacted.push(StreamMessage {
                        role: "system".to_string(),
                        content: Some(format!("【早前对话摘要】\n{digest}")),
                        tool_calls: None,
                        tool_call_id: None,
                    });
                    compacted.extend(kept);
                    messages = compacted;
                }
            }
        }

        // 跨会话记忆检索：嵌入用户文本取 top-k 相关记忆，作为 system 消息注入。
        // best-effort：检索失败只告警，不阻塞发送。
        let mut memory_context: Option<String> = None;
        if let (Some(memory), Some(embedder)) = (&memory, &embedder) {
            let query = text.clone();
            let embedder = Arc::clone(embedder);
            let embedding = match spawn_blocking(move || embedder.embed_text(&query)).await {
                Ok(Ok(embedding)) => Some(embedding),
                Ok(Err(error)) => {
                    tracing::warn!(%session_id, %error, "memory embed failed; skip retrieval");
                    None
                }
                Err(error) => {
                    tracing::warn!(
                        %session_id,
                        %error,
                        "memory embed task join failed; skip retrieval"
                    );
                    None
                }
            };
            if let Some(embedding) = embedding {
                match memory.search_memories(&embedding, MEMORY_TOP_K) {
                    Ok(hits) if !hits.is_empty() => {
                        memory_context = Some(
                            hits.iter()
                                .map(|hit| hit.content.clone())
                                .collect::<Vec<_>>()
                                .join("\n"),
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(%session_id, %error, "memory search failed; skip retrieval");
                    }
                }
            }
        }

        // 知识库检索：嵌入用户文本取 top-k 文档分块，作为 system 上下文注入。
        // best-effort：检索失败或知识库为空时只告警，不阻塞发送。
        let mut knowledge_context: Option<String> = None;
        if let (Some(knowledge), Some(embedder)) = (&knowledge, &embedder) {
            let empty = match knowledge.count_documents() {
                Ok(count) => count == 0,
                Err(_) => true,
            };
            if !empty {
                let query = text.clone();
                let embedder = Arc::clone(embedder);
                let embedding = match spawn_blocking(move || embedder.embed_text(&query)).await {
                    Ok(Ok(embedding)) => Some(embedding),
                    Ok(Err(error)) => {
                        tracing::warn!(%session_id, %error, "knowledge embed failed; skip retrieval");
                        None
                    }
                    Err(error) => {
                        tracing::warn!(
                            %session_id,
                            %error,
                            "knowledge embed task join failed; skip retrieval"
                        );
                        None
                    }
                };
                if let Some(embedding) = embedding {
                    match knowledge.search_chunks(&embedding, KNOWLEDGE_TOP_K) {
                        Ok(hits) if !hits.is_empty() => {
                            knowledge_context = Some(
                                hits.iter()
                                    .map(|hit| format!("[来自《{}》]\n{}", hit.name, hit.content))
                                    .collect::<Vec<_>>()
                                    .join("\n\n"),
                            );
                        }
                        Ok(_) => {}
                        Err(error) => {
                            tracing::warn!(%session_id, %error, "knowledge search failed; skip retrieval");
                        }
                    }
                }
            }
        }

        let user_text = text.clone();
        if let Some(context) = memory_context {
            messages.insert(0, StreamMessage {
                role: "system".to_string(),
                content: Some(format!(
                    "以下是此前对话中检索到的相关记忆（可能有过时或重复，仅在与当前问题相关时参考）：\n{context}"
                )),
                tool_calls: None,
                tool_call_id: None,
            });
        }
        if let Some(context) = knowledge_context {
            messages.insert(0, StreamMessage {
                role: "system".to_string(),
                content: Some(format!(
                    "以下是知识库中检索到的相关资料（公司产品 / 客户文档，仅在与当前问题相关时参考，不要编造资料中不存在的信息）：\n{context}"
                )),
                tool_calls: None,
                tool_call_id: None,
            });
        }
        // 系统操作指引：注入内置 Skill 知识，让 AI 了解页面 / 设置 / 操作路径。
        // 放到消息最前（system_prompt 会把所有 system 消息 join，OpenAI 与 Anthropic 路径均生效）。
        if let Some(skills) = &skills {
            let guide = skills.guide_text();
            if !guide.trim().is_empty() {
                messages.insert(
                    0,
                    StreamMessage {
                        role: "system".to_string(),
                        content: Some(format!("【系统操作指南】\n{guide}")),
                        tool_calls: None,
                        tool_call_id: None,
                    },
                );
            }
        }
        messages.push(StreamMessage {
            role: "user".to_string(),
            content: Some(user_text.clone()),
            tool_calls: None,
            tool_call_id: None,
        });

        // 已配置持久化：用户消息先落库（即使后续流式失败也不丢）。
        if let Some(store) = store {
            if let Err(error) = store.save_message(SaveChatMessage {
                id: Uuid::new_v4().to_string(),
                session_id: session_id.clone(),
                role: "user".to_string(),
                content: user_text,
                thinking: None,
                tools_json: None,
            }) {
                tracing::warn!(%session_id, %error, "persist user message failed");
            }
        }

        // 前端已预建 assistant 占位消息（带自己的 id）；复用该 id 使 token 落到占位上。
        let message_id = message_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let started = Instant::now();
        let mut assistant_content = String::new();
        let mut assistant_thinking = String::new();
        let mut assistant_tools: Vec<Value> = Vec::new();
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
                            assistant_thinking.push_str(&reasoning);
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
                            assistant_content.push_str(&text);
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
                    occurred_at: now_secs_string(),
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
                assistant_tools.push(json!({
                    "name": name.clone(),
                    "arguments": serde_json::to_string(&args_value).unwrap_or_default(),
                    "ok": ok,
                    "result": result_text.clone(),
                }));

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

        // 已配置持久化：落库完成态的 assistant 消息（含推理与工具步骤）。
        if let Some(store) = store {
            let tools_json = serde_json::to_string(&assistant_tools).unwrap_or_default();
            let has_content = !assistant_content.is_empty()
                || !assistant_thinking.is_empty()
                || !assistant_tools.is_empty();
            if has_content {
                if let Err(error) = store.save_message(SaveChatMessage {
                    id: message_id.clone(),
                    session_id: session_id.clone(),
                    role: "assistant".to_string(),
                    content: assistant_content,
                    thinking: Some(assistant_thinking).filter(|value| !value.is_empty()),
                    tools_json: Some(tools_json).filter(|value| !value.is_empty() && value != "[]"),
                }) {
                    tracing::warn!(%session_id, %message_id, %error, "persist assistant message failed");
                }
            }
        }

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
    emitter: &dyn ChatUIEmitter,
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
        occurred_at: now_secs_string(),
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
    use ports::chat::{ChatMessageRecord, ChatSessionRecord, ChatStore, SaveChatMessage};
    use ports::repository::StoreError;
    use serde_json::{json, Value};

    use super::merge_tool_call;
    use super::parse_history;
    use crate::emit::{ChatUIEmitter, NoopChatUIEmitter};
    use crate::tool::{ChatTool, ChatToolCaller};
    use crate::SendChat;
    use agent::llm::{LlmClient, StreamMessage, ToolCallDelta};
    use ports::knowledge::{KnowledgeChunkHit, KnowledgeDocumentRecord, KnowledgeStore};

    /// 内存版 `ChatStore`，用于验证 send_chat 的落库行为。
    #[derive(Default)]
    struct FakeStore {
        messages: Arc<Mutex<Vec<ChatMessageRecord>>>,
    }

    impl ChatStore for FakeStore {
        fn list_sessions(&self) -> Result<Vec<ChatSessionRecord>, StoreError> {
            Ok(Vec::new())
        }
        fn get_session(&self, _id: &str) -> Result<Option<ChatSessionRecord>, StoreError> {
            Ok(None)
        }
        fn create_session(&self, _id: &str, _title: &str) -> Result<ChatSessionRecord, StoreError> {
            unimplemented!("not used in send_chat tests")
        }
        fn rename_session(&self, _id: &str, _title: &str) -> Result<ChatSessionRecord, StoreError> {
            unimplemented!("not used in send_chat tests")
        }
        fn delete_session(&self, _id: &str) -> Result<(), StoreError> {
            Ok(())
        }
        fn load_messages(&self, _session_id: &str) -> Result<Vec<ChatMessageRecord>, StoreError> {
            Ok(self.messages.lock().unwrap().clone())
        }
        fn save_message(&self, input: SaveChatMessage) -> Result<ChatMessageRecord, StoreError> {
            let mut messages = self.messages.lock().unwrap();
            let record = ChatMessageRecord {
                id: input.id,
                session_id: input.session_id,
                role: input.role,
                content: input.content,
                thinking: input.thinking,
                tools_json: input.tools_json,
                seq: messages.len() as i64,
                created_at: 0,
            };
            messages.push(record.clone());
            Ok(record)
        }
        fn get_summary_state(&self, _session_id: &str) -> Result<Option<String>, StoreError> {
            Ok(None)
        }
        fn set_summary_state(&self, _session_id: &str, _json: &str) -> Result<(), StoreError> {
            Ok(())
        }
    }

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
                    // 读完请求头后按 Content-Length 继续读完整请求体（大请求体可能分多次到达）。
                    let text = String::from_utf8_lossy(&request);
                    if let Some(head_end) = text.find("\r\n\r\n") {
                        let content_length = text
                            .lines()
                            .find_map(|line| {
                                let lower = line.to_ascii_lowercase();
                                lower
                                    .strip_prefix("content-length:")
                                    .and_then(|value| value.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        if request.len() >= head_end + 4 + content_length {
                            break;
                        }
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

    impl ChatUIEmitter for RecordingEmitter {
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
                messages_json: None,
                text: "有几张表？".to_string(),
                trace_id: None,
                message_id: Some("msg-1".to_string()),
            },
            Some(Arc::new(MockTools)),
            None,
            None,
            None,
            None,
            None,
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
            &NoopChatUIEmitter,
            ChatIpcSendRequest {
                session_id: "sess-2".to_string(),
                messages_json: None,
                text: "你好".to_string(),
                trace_id: None,
                message_id: Some("msg-2".to_string()),
            },
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;
        let _ = emitter;
        Ok(())
    }

    #[tokio::test]
    async fn persists_user_and_assistant_messages() -> Result<(), Box<dyn std::error::Error>> {
        let round_text = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"回复内容\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{}}]}\n",
            "data: [DONE]\n",
            "\n",
        );
        let (base_url, _requests) = mock_llm_rounds(&[round_text])?;
        let client = LlmClient::new(llm_config(base_url))?;
        let store = FakeStore::default();

        SendChat::execute(
            &client,
            &NoopChatUIEmitter,
            ChatIpcSendRequest {
                session_id: "sess-3".to_string(),
                messages_json: None,
                text: "你好".to_string(),
                trace_id: None,
                message_id: Some("msg-3".to_string()),
            },
            None,
            None,
            Some(&store),
            None,
            None,
            None,
        )
        .await?;

        let messages = store.messages.lock().unwrap();
        assert_eq!(messages.len(), 2, "user + assistant persisted");
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "你好");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[1].content, "回复内容");
        Ok(())
    }

    #[tokio::test]
    async fn injects_system_guide_when_skills_provided() -> Result<(), Box<dyn std::error::Error>> {
        let round_text = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"好的\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{}}]}\n",
            "data: [DONE]\n",
            "\n",
        );
        let (base_url, requests) = mock_llm_rounds(&[round_text])?;
        let client = LlmClient::new(llm_config(base_url))?;
        let skills = Arc::new(agent::skills::system::system_registry());

        SendChat::execute(
            &client,
            &NoopChatUIEmitter,
            ChatIpcSendRequest {
                session_id: "sess-skills".to_string(),
                messages_json: None,
                text: "怎么配置 LLM".to_string(),
                trace_id: None,
                message_id: Some("msg-skills".to_string()),
            },
            None,
            Some(skills),
            None,
            None,
            None,
            None,
        )
        .await?;

        let raw = requests.recv_timeout(std::time::Duration::from_secs(2))?;
        let body = raw.split("\r\n\r\n").nth(1).unwrap_or_default();
        let parsed: Value = serde_json::from_str(body)?;
        let messages = parsed["messages"].as_array().expect("messages array");
        let guide = messages
            .iter()
            .find(|message| message["role"] == "system")
            .and_then(|message| message["content"].as_str())
            .unwrap_or_default();
        assert!(
            guide.contains("系统操作指南"),
            "system guide injected: {guide}"
        );
        assert!(
            guide.contains("navigate_page"),
            "page map mentions navigate_page: {guide}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn no_guide_injected_without_skills() -> Result<(), Box<dyn std::error::Error>> {
        let round_text = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"好的\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{}}]}\n",
            "data: [DONE]\n",
            "\n",
        );
        let (base_url, requests) = mock_llm_rounds(&[round_text])?;
        let client = LlmClient::new(llm_config(base_url))?;

        SendChat::execute(
            &client,
            &NoopChatUIEmitter,
            ChatIpcSendRequest {
                session_id: "sess-noskill".to_string(),
                messages_json: None,
                text: "你好".to_string(),
                trace_id: None,
                message_id: Some("msg-noskill".to_string()),
            },
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

        let raw = requests.recv_timeout(std::time::Duration::from_secs(2))?;
        let body = raw.split("\r\n\r\n").nth(1).unwrap_or_default();
        let parsed: Value = serde_json::from_str(body)?;
        let messages = parsed["messages"].as_array().expect("messages array");
        let guide = messages
            .iter()
            .find(|message| message["role"] == "system")
            .and_then(|message| message["content"].as_str())
            .unwrap_or_default();
        assert!(
            !guide.contains("系统操作指南"),
            "no guide when skills is None: {guide}"
        );
        Ok(())
    }

    /// 假嵌入器：返回固定 512 维向量，供知识库检索测试使用。
    struct FakeEmbedder;

    impl agent::embedding::Embedder for FakeEmbedder {
        fn dims(&self) -> usize {
            512
        }
        fn preload(&self) -> Result<(), agent::embedding::EmbeddingError> {
            Ok(())
        }
        fn embed_texts(
            &self,
            texts: &[String],
        ) -> Result<Vec<Vec<f32>>, agent::embedding::EmbeddingError> {
            Ok(texts.iter().map(|_| vec![0.5f32; 512]).collect())
        }
    }

    /// 内存版 `KnowledgeStore`，用于验证知识库检索注入。
    #[derive(Default)]
    struct FakeKnowledgeStore {
        chunks: Mutex<Vec<KnowledgeChunkHit>>,
    }

    impl FakeKnowledgeStore {
        fn seed(content: &str) -> Arc<Self> {
            let store = FakeKnowledgeStore {
                chunks: Mutex::new(vec![KnowledgeChunkHit {
                    doc_id: "doc-1".to_string(),
                    name: "产品手册.md".to_string(),
                    content: content.to_string(),
                    distance: 0.1,
                }]),
            };
            Arc::new(store)
        }
    }

    impl KnowledgeStore for FakeKnowledgeStore {
        fn create_document(
            &self,
            _id: &str,
            _name: &str,
            _source_type: &str,
        ) -> Result<KnowledgeDocumentRecord, StoreError> {
            Ok(KnowledgeDocumentRecord {
                id: "doc-1".to_string(),
                name: "产品手册.md".to_string(),
                source_type: "md".to_string(),
                status: "ready".to_string(),
                chunk_count: 1,
                created_at: 0,
                updated_at: 0,
            })
        }
        fn insert_chunk(
            &self,
            _doc_id: &str,
            _content: &str,
            _seq: i64,
            _embedding: &[f32],
        ) -> Result<(), StoreError> {
            Ok(())
        }
        fn finish_document(&self, _id: &str, _chunk_count: i64) -> Result<(), StoreError> {
            Ok(())
        }
        fn search_chunks(
            &self,
            _query_embedding: &[f32],
            _k: usize,
        ) -> Result<Vec<KnowledgeChunkHit>, StoreError> {
            Ok(self.chunks.lock().unwrap().clone())
        }
        fn list_documents(&self) -> Result<Vec<KnowledgeDocumentRecord>, StoreError> {
            Ok(Vec::new())
        }
        fn delete_document(&self, _id: &str) -> Result<(), StoreError> {
            Ok(())
        }
        fn count_documents(&self) -> Result<usize, StoreError> {
            Ok(self.chunks.lock().unwrap().len())
        }
    }

    #[tokio::test]
    async fn injects_knowledge_context_when_store_has_documents(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let round_text = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"好的\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{}}]}\n",
            "data: [DONE]\n",
            "\n",
        );
        let (base_url, requests) = mock_llm_rounds(&[round_text])?;
        let client = LlmClient::new(llm_config(base_url))?;
        let knowledge = FakeKnowledgeStore::seed("本产品支持多语言邮件模板。");
        let embedder: Arc<dyn agent::embedding::Embedder> = Arc::new(FakeEmbedder);

        SendChat::execute(
            &client,
            &NoopChatUIEmitter,
            ChatIpcSendRequest {
                session_id: "sess-kb".to_string(),
                messages_json: None,
                text: "邮件模板支持哪些语言？".to_string(),
                trace_id: None,
                message_id: Some("msg-kb".to_string()),
            },
            None,
            None,
            None,
            None,
            Some(embedder),
            Some(knowledge),
        )
        .await?;

        let raw = requests.recv_timeout(std::time::Duration::from_secs(2))?;
        let body = raw.split("\r\n\r\n").nth(1).unwrap_or_default();
        let parsed: Value = serde_json::from_str(body)?;
        let messages = parsed["messages"].as_array().expect("messages array");
        let system_text = messages
            .iter()
            .filter(|message| message["role"] == "system")
            .map(|message| message["content"].as_str().unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            system_text.contains("知识库"),
            "knowledge context injected: {system_text}"
        );
        assert!(
            system_text.contains("多语言邮件模板"),
            "knowledge content present: {system_text}"
        );
        Ok(())
    }
}
