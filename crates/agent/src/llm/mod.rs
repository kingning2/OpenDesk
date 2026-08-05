//! Rust 直连 LLM 的最小 HTTP 客户端。

use std::time::Duration;

use reqwest::{Client as HttpClient, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::mpsc;

use crate::prompt::Prompt;

const OPENAI_BASE: &str = "https://api.openai.com/v1";
const ANTHROPIC_BASE: &str = "https://api.anthropic.com";
const DEEPSEEK_BASE: &str = "https://api.deepseek.com";
const DOUBAO_BASE: &str = "https://ark.cn-beijing.volces.com/api/v3";
const KIMI_BASE: &str = "https://api.moonshot.cn/v1";
const OLLAMA_BASE: &str = "http://127.0.0.1:11434/v1";

/// LLM HTTP 协议策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// OpenAI Chat Completions 兼容协议。
    OpenAiCompatible,
    /// Anthropic Messages 协议。
    Anthropic,
}

/// 单个 LLM 调用配置。
#[derive(Debug, Clone)]
pub struct Config {
    /// Provider 名称。
    pub provider: String,
    /// 可选自定义基础 URL。
    pub base_url: Option<String>,
    /// 模型 ID。
    pub model_id: String,
    /// API Key；本地兼容端点允许为空。
    pub api_key: String,
}

/// 对话历史中的一条消息。
///
/// `content: None` 表示 assistant 工具调用消息（OpenAI 协议要求此时带 `content:null`）；
/// `tool_call_id` 用于 `tool` 角色结果消息回指对应调用。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamMessage {
    /// 消息角色：`system` / `user` / `assistant` / `tool`。
    pub role: String,
    /// 消息正文；工具调用消息为 `None`。
    pub content: Option<String>,
    /// assistant 消息携带的工具调用。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallMsg>>,
    /// `tool` 结果消息引用的工具调用 id。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// assistant 消息中的一个工具调用（语义形态，发送前翻译为 OpenAI 线格式）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallMsg {
    /// 工具调用 id（供 `tool` 结果消息引用）。
    pub id: String,
    /// 工具名。
    pub name: String,
    /// 工具参数（JSON 对象）。
    pub arguments: Value,
}

/// 可注入请求的工具定义。
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionTool {
    /// 工具名。
    pub name: String,
    /// 工具说明。
    pub description: String,
    /// 参数 JSON Schema 对象。
    pub parameters: Value,
}

/// 流式响应中的一段增量：正文文本或推理（thinking）内容，二者可并存。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StreamDelta {
    /// 正文增量（assistant 回复内容）。
    pub text: Option<String>,
    /// 推理增量（如 DeepSeek `reasoning_content`、Anthropic `thinking`）。
    pub reasoning: Option<String>,
    /// 工具调用增量片段（OpenAI 按 `index` 分段推送）。
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

/// 工具调用增量片段：同一 `index` 的多个片段按 id/name/arguments 顺序拼接。
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCallDelta {
    /// 该次回复中第几个工具调用（0-based）。
    pub index: usize,
    /// 首个片段携带调用 id。
    pub id: Option<String>,
    /// 首个片段携带工具名。
    pub name: Option<String>,
    /// 参数 JSON 字符串的增量片段。
    pub arguments: Option<String>,
}

/// 多轮流式对话请求。
#[derive(Debug, Clone)]
pub struct StreamRequest {
    /// 按时间顺序的对话历史（含本轮用户消息）。
    pub messages: Vec<StreamMessage>,
    /// 采样温度。
    pub temperature: f32,
    /// 可调用工具定义；空则不在请求中携带 `tools`。
    pub tools: Vec<FunctionTool>,
}

/// LLM 请求错误。
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP 客户端或网络错误。
    #[error("LLM HTTP 请求失败: {0}")]
    Http(#[from] reqwest::Error),
    /// 服务端返回非成功状态。
    #[error("LLM API 返回 HTTP {status}: {message}")]
    Api {
        /// HTTP 状态码。
        status: StatusCode,
        /// 截断后的响应正文。
        message: String,
    },
    /// 响应 JSON 不符合协议。
    #[error("LLM 响应格式无效: {0}")]
    InvalidResponse(#[from] serde_json::Error),
    /// 响应中没有可用文本。
    #[error("LLM 响应没有文本内容")]
    EmptyResponse,
}

/// 绑定配置与协议策略的 LLM 客户端。
#[derive(Clone)]
pub struct LlmClient {
    http: HttpClient,
    config: Config,
    strategy: Strategy,
    base_url: String,
}

impl LlmClient {
    /// 按 Provider 创建客户端；未知 Provider 按 OpenAI-compatible 处理。
    ///
    /// # Errors
    ///
    /// HTTP 客户端初始化失败时返回 [`Error::Http`]。
    pub fn new(config: Config) -> Result<Self, Error> {
        let (strategy, default_base) = strategy_and_default(&config.provider);
        let configured_base = config
            .base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let base_url = match configured_base {
            Some(value) => value,
            None => default_base,
        }
        .trim_end_matches('/')
        .to_string();
        let http = HttpClient::builder()
            .timeout(Duration::from_secs(60))
            .build()?;
        Ok(Self {
            http,
            config,
            strategy,
            base_url,
        })
    }

    /// 返回当前客户端采用的协议策略。
    pub fn strategy(&self) -> Strategy {
        self.strategy
    }

    /// 发送一次文本对话并返回助手文本。
    ///
    /// # Errors
    ///
    /// 网络失败、非成功状态、无效 JSON 或空文本响应时返回错误。
    pub async fn complete(&self, prompt: &Prompt<'_>) -> Result<String, Error> {
        match self.strategy {
            Strategy::OpenAiCompatible => {
                let request = OpenAiRequest {
                    model: &self.config.model_id,
                    temperature: prompt.temperature,
                    messages: [
                        OpenAiMessage {
                            role: "system",
                            content: prompt.system,
                        },
                        OpenAiMessage {
                            role: "user",
                            content: prompt.user,
                        },
                    ],
                };
                let response: OpenAiResponse = self
                    .send_json(
                        self.auth(
                            self.http
                                .post(versioned_endpoint(&self.base_url, "chat/completions")),
                        ),
                        &request,
                    )
                    .await?;
                response
                    .choices
                    .into_iter()
                    .next()
                    .map(|choice| choice.message.content.trim().to_string())
                    .filter(|text| !text.is_empty())
                    .ok_or(Error::EmptyResponse)
            }
            Strategy::Anthropic => {
                let request = AnthropicRequest {
                    model: &self.config.model_id,
                    max_tokens: 4096,
                    temperature: prompt.temperature,
                    system: prompt.system,
                    messages: [AnthropicMessage {
                        role: "user",
                        content: prompt.user,
                    }],
                };
                let response: AnthropicResponse = self
                    .send_json(
                        self.auth(
                            self.http
                                .post(versioned_endpoint(&self.base_url, "messages")),
                        ),
                        &request,
                    )
                    .await?;
                let text = response
                    .content
                    .into_iter()
                    .filter_map(|block| block.text)
                    .map(|text| text.trim().to_string())
                    .filter(|text| !text.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
                if text.is_empty() {
                    Err(Error::EmptyResponse)
                } else {
                    Ok(text)
                }
            }
        }
    }

    /// 以流式方式发送一次多轮对话，返回逐字增量。
    ///
    /// 返回的 channel receiver 逐条产出 [`Result<StreamDelta, Error>`]：`Ok` 为一段增量
    /// （正文与/或推理文本），`Err` 表示流中断。流结束（或接收端被丢弃）后 channel 关闭。
    ///
    /// # Errors
    ///
    /// 网络失败或非成功状态时返回错误。
    pub async fn stream(
        &self,
        request: &StreamRequest,
    ) -> Result<mpsc::Receiver<Result<StreamDelta, Error>>, Error> {
        let strategy = self.strategy;
        let request_builder = match strategy {
            Strategy::OpenAiCompatible => {
                let messages = openai_wire_messages(&request.messages);
                let tools: Vec<OpenAiWireTool<'_>> = request
                    .tools
                    .iter()
                    .map(|tool| OpenAiWireTool {
                        kind: "function",
                        function: OpenAiWireToolFunction {
                            name: tool.name.as_str(),
                            description: tool.description.as_str(),
                            parameters: &tool.parameters,
                        },
                    })
                    .collect();
                let body = OpenAiStreamRequest {
                    model: &self.config.model_id,
                    temperature: request.temperature,
                    stream: true,
                    messages: &messages,
                    tools,
                };
                self.http
                    .post(versioned_endpoint(&self.base_url, "chat/completions"))
                    .timeout(STREAM_TIMEOUT)
                    .json(&body)
            }
            Strategy::Anthropic => {
                let system = system_prompt(&request.messages);
                let messages: Vec<StreamMessage> = request
                    .messages
                    .iter()
                    .filter(|message| message.role != "system")
                    .cloned()
                    .collect();
                let body = AnthropicStreamRequest {
                    model: &self.config.model_id,
                    max_tokens: 4096,
                    temperature: request.temperature,
                    stream: true,
                    system: &system,
                    messages: &messages,
                };
                self.http
                    .post(versioned_endpoint(&self.base_url, "messages"))
                    .timeout(STREAM_TIMEOUT)
                    .json(&body)
            }
        };
        let response = self.send(self.auth(request_builder)).await?;
        let (sender, receiver) = mpsc::channel(64);
        tokio::spawn(pump_stream(response, strategy, sender));
        Ok(receiver)
    }

    /// 通过 Provider 的模型列表接口测试连接。
    ///
    /// # Errors
    ///
    /// 网络失败、非成功状态或无效 JSON 时返回错误。
    pub async fn test_connection(&self) -> Result<(), Error> {
        let request = self.auth(self.http.get(versioned_endpoint(&self.base_url, "models")));
        let _: Value = self.send(request).await?.json().await?;
        Ok(())
    }

    fn auth(&self, request: RequestBuilder) -> RequestBuilder {
        match self.strategy {
            Strategy::OpenAiCompatible => {
                request.bearer_auth(if self.config.api_key.trim().is_empty() {
                    "opendesk"
                } else {
                    self.config.api_key.trim()
                })
            }
            Strategy::Anthropic => request
                .header("x-api-key", self.config.api_key.trim())
                .header("anthropic-version", "2023-06-01"),
        }
    }

    async fn send_json<T, R>(&self, request: RequestBuilder, body: &T) -> Result<R, Error>
    where
        T: Serialize + ?Sized,
        R: for<'de> Deserialize<'de>,
    {
        let bytes = self.send(request.json(body)).await?.bytes().await?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    async fn send(&self, request: RequestBuilder) -> Result<reqwest::Response, Error> {
        let response = request.send().await?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let message = response.text().await?;
        Err(Error::Api {
            status,
            message: message.chars().take(500).collect(),
        })
    }
}

fn strategy_and_default(provider: &str) -> (Strategy, &'static str) {
    match provider.trim().to_ascii_lowercase().as_str() {
        "anthropic" => (Strategy::Anthropic, ANTHROPIC_BASE),
        "deepseek" => (Strategy::OpenAiCompatible, DEEPSEEK_BASE),
        "doubao" | "ark" | "volcengine" => (Strategy::OpenAiCompatible, DOUBAO_BASE),
        "kimi" | "moonshot" => (Strategy::OpenAiCompatible, KIMI_BASE),
        "ollama" => (Strategy::OpenAiCompatible, OLLAMA_BASE),
        "openai" | "openai_compatible" => (Strategy::OpenAiCompatible, OPENAI_BASE),
        _ => (Strategy::OpenAiCompatible, OPENAI_BASE),
    }
}

fn versioned_endpoint(base_url: &str, resource: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let has_version = base.rsplit('/').next().is_some_and(|segment| {
        segment.strip_prefix('v').is_some_and(|version| {
            !version.is_empty() && version.chars().all(|ch| ch.is_ascii_digit())
        })
    });
    if has_version {
        format!("{base}/{resource}")
    } else {
        format!("{base}/v1/{resource}")
    }
}

/// 流式请求的超时（覆盖客户端的 60 秒总超时，避免长回复被截断）。
const STREAM_TIMEOUT: Duration = Duration::from_secs(600);

/// 提取对话历史中的 system 消息作为顶层提示（Anthropic 协议要求）。
fn system_prompt(messages: &[StreamMessage]) -> String {
    messages
        .iter()
        .filter(|message| message.role == "system")
        .map(|message| message.content.as_deref().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

/// 把语义消息翻译为 OpenAI Chat Completions 线格式（工具调用消息、tool 结果消息）。
fn openai_wire_messages(messages: &[StreamMessage]) -> Vec<OpenAiWireMessage<'_>> {
    messages
        .iter()
        .map(|message| OpenAiWireMessage {
            role: message.role.as_str(),
            content: message.content.as_deref(),
            tool_calls: message.tool_calls.as_ref().map(|calls| {
                calls
                    .iter()
                    .map(|call| OpenAiWireToolCall {
                        id: call.id.as_str(),
                        kind: "function",
                        function: OpenAiWireCallFunction {
                            name: call.name.as_str(),
                            // OpenAI 要求 function.arguments 是 JSON 字符串。
                            arguments: serde_json::to_string(&call.arguments).unwrap_or_default(),
                        },
                    })
                    .collect()
            }),
            tool_call_id: message.tool_call_id.as_deref(),
        })
        .collect()
}

/// 按协议策略解析一行 SSE 文本，返回增量（正文与/或推理；无增量返回 `None`）。
fn parse_sse_delta(line: &str, strategy: Strategy) -> Option<StreamDelta> {
    match strategy {
        Strategy::OpenAiCompatible => parse_openai_sse_delta(line),
        Strategy::Anthropic => parse_anthropic_sse_delta(line),
    }
}

/// 解析 OpenAI Chat Completions 流式行（`data: {...}`），提取正文、推理与工具调用增量。
fn parse_openai_sse_delta(line: &str) -> Option<StreamDelta> {
    let data = line.strip_prefix("data:")?.trim();
    if data == "[DONE]" {
        return None;
    }
    let value: Value = serde_json::from_str(data).ok()?;
    let text = value
        .pointer("/choices/0/delta/content")
        .and_then(Value::as_str)
        .map(str::to_string);
    let reasoning = value
        .pointer("/choices/0/delta/reasoning_content")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            value
                .pointer("/choices/0/delta/reasoning")
                .and_then(Value::as_str)
                .map(str::to_string)
        });
    let tool_calls = value
        .pointer("/choices/0/delta/tool_calls")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let index = item.get("index").and_then(Value::as_u64)? as usize;
                    let id = item.get("id").and_then(Value::as_str).map(str::to_string);
                    let name = item
                        .pointer("/function/name")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    let arguments = item
                        .pointer("/function/arguments")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    Some(ToolCallDelta {
                        index,
                        id,
                        name,
                        arguments,
                    })
                })
                .collect()
        })
        .filter(|items: &Vec<ToolCallDelta>| !items.is_empty());
    (text.is_some() || reasoning.is_some() || tool_calls.is_some()).then_some(StreamDelta {
        text,
        reasoning,
        tool_calls,
    })
}

/// 解析 Anthropic Messages 流式行（`data: {...}`），提取正文与推理增量。
fn parse_anthropic_sse_delta(line: &str) -> Option<StreamDelta> {
    let data = line.strip_prefix("data:")?.trim();
    let value: Value = serde_json::from_str(data).ok()?;
    if value.pointer("/type").and_then(Value::as_str) != Some("content_block_delta") {
        return None;
    }
    match value.pointer("/delta/type").and_then(Value::as_str) {
        Some("text_delta") => {
            let text = value
                .pointer("/delta/text")
                .and_then(Value::as_str)
                .map(str::to_string);
            Some(StreamDelta {
                text,
                reasoning: None,
                tool_calls: None,
            })
        }
        Some("thinking_delta") => {
            let reasoning = value
                .pointer("/delta/thinking")
                .and_then(Value::as_str)
                .map(str::to_string);
            Some(StreamDelta {
                text: None,
                reasoning,
                tool_calls: None,
            })
        }
        _ => None,
    }
}

/// 从响应体逐块读取并解析 SSE，把增量推入 channel；流出错时推入 `Err`。
async fn pump_stream(
    mut response: reqwest::Response,
    strategy: Strategy,
    sender: mpsc::Sender<Result<StreamDelta, Error>>,
) {
    let mut pending: Vec<u8> = Vec::new();
    loop {
        let chunk = match response.chunk().await {
            Ok(Some(chunk)) => chunk,
            Ok(None) => break,
            Err(error) => {
                let _ = sender.send(Err(error.into())).await;
                break;
            }
        };
        pending.extend_from_slice(&chunk);
        while let Some(position) = pending.iter().position(|byte| *byte == b'\n') {
            let raw: Vec<u8> = pending.drain(..=position).collect();
            let line = String::from_utf8_lossy(&raw);
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(delta) = parse_sse_delta(line, strategy) {
                if sender.send(Ok(delta)).await.is_err() {
                    return;
                }
            }
        }
    }
}

#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    temperature: f32,
    messages: [OpenAiMessage<'a>; 2],
}

#[derive(Serialize)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    #[serde(default)]
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
}

#[derive(Deserialize)]
struct OpenAiResponseMessage {
    #[serde(default)]
    content: String,
}

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    temperature: f32,
    system: &'a str,
    messages: [AnthropicMessage<'a>; 1],
}

#[derive(Serialize)]
struct AnthropicMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct OpenAiStreamRequest<'a> {
    model: &'a str,
    temperature: f32,
    stream: bool,
    messages: &'a [OpenAiWireMessage<'a>],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<OpenAiWireTool<'a>>,
}

/// OpenAI 线格式消息：`content:null` 表示 assistant 工具调用消息，`tool_call_id` 用于 tool 结果。
#[derive(Serialize)]
struct OpenAiWireMessage<'a> {
    role: &'a str,
    content: Option<&'a str>,
    #[serde(rename = "tool_calls", skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAiWireToolCall<'a>>>,
    #[serde(rename = "tool_call_id", skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<&'a str>,
}

/// assistant 消息中的工具调用（`function.arguments` 为 JSON 字符串）。
#[derive(Serialize)]
struct OpenAiWireToolCall<'a> {
    id: &'a str,
    #[serde(rename = "type")]
    kind: &'static str,
    function: OpenAiWireCallFunction<'a>,
}

#[derive(Serialize)]
struct OpenAiWireCallFunction<'a> {
    name: &'a str,
    arguments: String,
}

/// 请求中注入的工具定义。
#[derive(Serialize)]
struct OpenAiWireTool<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    function: OpenAiWireToolFunction<'a>,
}

#[derive(Serialize)]
struct OpenAiWireToolFunction<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a Value,
}

#[derive(Serialize)]
struct AnthropicStreamRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
    system: &'a str,
    messages: &'a [StreamMessage],
}

#[derive(Deserialize)]
struct AnthropicResponse {
    #[serde(default)]
    content: Vec<AnthropicContent>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    text: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc::{self, Receiver};
    use std::thread;
    use std::time::Duration;

    use super::{
        parse_anthropic_sse_delta, parse_openai_sse_delta, Config, Error, FunctionTool, LlmClient,
        Strategy, StreamDelta, StreamMessage, StreamRequest, ToolCallDelta, ToolCallMsg,
    };
    use crate::prompt::Prompt;
    use serde_json::json;

    fn mock_server(status: &str, body: &str) -> std::io::Result<(String, Receiver<String>)> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let (sender, receiver) = mpsc::channel();
        let status = status.to_string();
        let body = body.to_string();
        thread::spawn(move || {
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
                let text = String::from_utf8_lossy(&request);
                let Some(header_end) = text.find("\r\n\r\n") else {
                    continue;
                };
                let content_length: usize = text[..header_end]
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .and_then(|value| value.trim().parse::<usize>().ok())
                    })
                    .unwrap_or_default();
                if request.len() >= header_end + 4 + content_length {
                    break;
                }
            }
            let _ = write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = sender.send(String::from_utf8_lossy(&request).to_string());
        });
        Ok((format!("http://{address}"), receiver))
    }

    /// 返回一个按两段写入的 SSE 服务器，用于验证跨 chunk 的流式解析。
    fn mock_sse_server(body: &str) -> std::io::Result<(String, Receiver<String>)> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let (sender, receiver) = mpsc::channel();
        let body = body.to_string();
        thread::spawn(move || {
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
            let (first, second) = body.split_at(body.len() / 2);
            let _ = stream.write_all(first.as_bytes());
            let _ = stream.flush();
            thread::sleep(Duration::from_millis(20));
            let _ = stream.write_all(second.as_bytes());
            let _ = stream.flush();
            let _ = sender.send(String::from_utf8_lossy(&request).to_string());
        });
        Ok((format!("http://{address}"), receiver))
    }

    fn config(provider: &str, base_url: String) -> Config {
        Config {
            provider: provider.to_string(),
            base_url: Some(base_url),
            model_id: "test-model".to_string(),
            api_key: String::from("test-token"),
        }
    }

    #[tokio::test]
    async fn openai_chat_sends_compatible_request_and_parses_text(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (base_url, request) = mock_server(
            "200 OK",
            r#"{"choices":[{"message":{"content":" hello "}}]}"#,
        )?;
        let client = LlmClient::new(config("deepseek", base_url))?;

        let text = client.complete(&Prompt::new("system", "user")).await?;
        let request = request.recv()?;

        assert_eq!(text, "hello");
        assert!(request.contains("POST /v1/chat/completions"));
        assert!(request.contains("\"role\":\"system\""));
        Ok(())
    }

    #[tokio::test]
    async fn anthropic_chat_sends_messages_request_and_joins_text_blocks(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (base_url, request) = mock_server(
            "200 OK",
            r#"{"content":[{"type":"text","text":"one"},{"type":"text","text":"two"}]}"#,
        )?;
        let client = LlmClient::new(config("anthropic", base_url))?;

        let text = client.complete(&Prompt::new("system", "user")).await?;
        let request = request.recv()?;

        assert_eq!(client.strategy(), Strategy::Anthropic);
        assert_eq!(text, "one\ntwo");
        assert!(request.contains("x-api-key: test-token"));
        assert!(request.contains("\"max_tokens\":4096"));
        Ok(())
    }

    #[tokio::test]
    async fn chat_returns_api_error_for_non_success_status(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (base_url, _) = mock_server("401 Unauthorized", r#"{"error":"bad key"}"#)?;
        let client = LlmClient::new(config("openai", base_url))?;
        let result = client.complete(&Prompt::new("system", "user")).await;

        assert!(matches!(result, Err(Error::Api { .. })));
        Ok(())
    }

    #[tokio::test]
    async fn chat_returns_empty_response_error_when_text_is_missing(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (base_url, _) = mock_server("200 OK", r#"{"choices":[]}"#)?;
        let client = LlmClient::new(config("openai", base_url))?;
        let result = client.complete(&Prompt::new("system", "user")).await;

        assert!(matches!(result, Err(Error::EmptyResponse)));
        Ok(())
    }

    #[test]
    fn openai_delta_parsing() {
        assert_eq!(
            parse_openai_sse_delta(r#"data: {"choices":[{"delta":{"content":"Hi"}}]}"#),
            Some(StreamDelta {
                text: Some("Hi".to_string()),
                reasoning: None,
                tool_calls: None,
            })
        );
        assert_eq!(
            parse_openai_sse_delta(
                r#"data: {"choices":[{"delta":{"reasoning_content":"Let me"}}]}"#
            ),
            Some(StreamDelta {
                text: None,
                reasoning: Some("Let me".to_string()),
                tool_calls: None,
            })
        );
        assert_eq!(
            parse_openai_sse_delta(r#"data: {"choices":[{"delta":{"reasoning":"Fallback"}}]}"#),
            Some(StreamDelta {
                text: None,
                reasoning: Some("Fallback".to_string()),
                tool_calls: None,
            })
        );
        assert_eq!(parse_openai_sse_delta("data: [DONE]"), None);
        assert_eq!(parse_openai_sse_delta("event: message_start"), None);
        assert_eq!(parse_openai_sse_delta("data: {bad json"), None);
    }

    #[test]
    fn openai_delta_parses_tool_calls() {
        assert_eq!(
            parse_openai_sse_delta(
                r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"list_tables","arguments":"{\"db\":"}}]}}]}"#
            ),
            Some(StreamDelta {
                text: None,
                reasoning: None,
                tool_calls: Some(vec![ToolCallDelta {
                    index: 0,
                    id: Some("call_1".to_string()),
                    name: Some("list_tables".to_string()),
                    arguments: Some("{\"db\":".to_string()),
                }]),
            })
        );
        assert_eq!(
            parse_openai_sse_delta(
                r#"data: {"choices":[{"delta":{"tool_calls":[{"index":1,"function":{"arguments":"\"opendesk\"}"}}]}}]}"#
            ),
            Some(StreamDelta {
                text: None,
                reasoning: None,
                tool_calls: Some(vec![ToolCallDelta {
                    index: 1,
                    id: None,
                    name: None,
                    arguments: Some("\"opendesk\"}".to_string()),
                }]),
            })
        );
    }

    #[test]
    fn anthropic_delta_parsing() {
        assert_eq!(
            parse_anthropic_sse_delta(
                r#"data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"Yo"}}"#
            ),
            Some(StreamDelta {
                text: Some("Yo".to_string()),
                reasoning: None,
                tool_calls: None,
            })
        );
        assert_eq!(
            parse_anthropic_sse_delta(
                r#"data: {"type":"content_block_delta","delta":{"type":"thinking_delta","thinking":"So"}}"#
            ),
            Some(StreamDelta {
                text: None,
                reasoning: Some("So".to_string()),
                tool_calls: None,
            })
        );
        assert_eq!(
            parse_anthropic_sse_delta(r#"data: {"type":"content_block_start","index":0}"#),
            None
        );
        assert_eq!(parse_anthropic_sse_delta("event: content_block_stop"), None);
    }

    #[tokio::test]
    async fn stream_openai_emits_deltas_in_order() -> Result<(), Box<dyn std::error::Error>> {
        let sse = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{}}]}\n",
            "data: [DONE]\n",
            "\n",
        );
        let (base_url, request) = mock_sse_server(sse)?;
        let client = LlmClient::new(config("deepseek", base_url))?;
        let mut receiver = client
            .stream(&StreamRequest {
                messages: vec![StreamMessage {
                    role: "user".to_string(),
                    content: Some("hi".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                }],
                temperature: 0.0,
                tools: vec![],
            })
            .await?;

        let mut deltas = Vec::new();
        while let Some(item) = receiver.recv().await {
            let delta = item?;
            if let Some(text) = delta.text {
                deltas.push(text);
            }
        }
        assert_eq!(deltas, vec!["Hel", "lo"]);

        let request = request.recv()?;
        assert!(request.contains("\"stream\":true"));
        assert!(request.contains("\"role\":\"user\""));
        Ok(())
    }

    #[tokio::test]
    async fn stream_openai_separates_reasoning_and_text() -> Result<(), Box<dyn std::error::Error>>
    {
        let sse = concat!(
            "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"First think\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"Answer\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"more\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{}}]}\n",
            "data: [DONE]\n",
            "\n",
        );
        let (base_url, _) = mock_sse_server(sse)?;
        let client = LlmClient::new(config("deepseek", base_url))?;
        let mut receiver = client
            .stream(&StreamRequest {
                messages: vec![StreamMessage {
                    role: "user".to_string(),
                    content: Some("hi".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                }],
                temperature: 0.0,
                tools: vec![],
            })
            .await?;

        let mut reasoning = Vec::new();
        let mut text = Vec::new();
        while let Some(item) = receiver.recv().await {
            let delta = item?;
            if let Some(value) = delta.reasoning {
                reasoning.push(value);
            }
            if let Some(value) = delta.text {
                text.push(value);
            }
        }
        assert_eq!(reasoning, vec!["First think", "more"]);
        assert_eq!(text, vec!["Answer"]);
        Ok(())
    }

    #[tokio::test]
    async fn stream_openai_sends_tools_and_tool_call_messages(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sse = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{}}]}\n",
            "data: [DONE]\n",
            "\n",
        );
        let (base_url, request) = mock_sse_server(sse)?;
        let client = LlmClient::new(config("deepseek", base_url))?;
        let mut receiver = client
            .stream(&StreamRequest {
                messages: vec![
                    StreamMessage {
                        role: "user".to_string(),
                        content: Some("how many tables?".to_string()),
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    StreamMessage {
                        role: "assistant".to_string(),
                        content: None,
                        tool_calls: Some(vec![ToolCallMsg {
                            id: "call_1".to_string(),
                            name: "list_tables".to_string(),
                            arguments: json!({ "db": "opendesk" }),
                        }]),
                        tool_call_id: None,
                    },
                    StreamMessage {
                        role: "tool".to_string(),
                        content: Some("[{\"tables\":[\"customer\"]}]".to_string()),
                        tool_calls: None,
                        tool_call_id: Some("call_1".to_string()),
                    },
                ],
                temperature: 0.0,
                tools: vec![FunctionTool {
                    name: "list_tables".to_string(),
                    description: "List tables".to_string(),
                    parameters: json!({ "type": "object", "properties": {} }),
                }],
            })
            .await?;
        while receiver.recv().await.is_some() {}

        let request = request.recv()?;
        assert!(request.contains(r#""tools":[{"type":"function""#));
        assert!(request.contains(r#""name":"list_tables""#));
        assert!(request.contains(r#""content":null"#));
        assert!(request.contains(r#""arguments":"{\"db\":\"opendesk\"}""#));
        assert!(request.contains(r#""role":"tool""#));
        assert!(request.contains(r#""tool_call_id":"call_1""#));
        Ok(())
    }

    #[tokio::test]
    async fn stream_anthropic_extracts_system_and_emits_text_deltas(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sse = concat!(
            "event: message_start\n",
            "data: {\"type\":\"message_start\"}\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Yo\"}}\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"lo\"}}\n",
            "event: message_stop\n",
            "data: {\"type\":\"message_stop\"}\n",
            "\n",
        );
        let (base_url, request) = mock_sse_server(sse)?;
        let client = LlmClient::new(config("anthropic", base_url))?;
        let mut receiver = client
            .stream(&StreamRequest {
                messages: vec![
                    StreamMessage {
                        role: "system".to_string(),
                        content: Some("be brief".to_string()),
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    StreamMessage {
                        role: "user".to_string(),
                        content: Some("hello".to_string()),
                        tool_calls: None,
                        tool_call_id: None,
                    },
                ],
                temperature: 0.0,
                tools: vec![],
            })
            .await?;

        let mut deltas = Vec::new();
        while let Some(item) = receiver.recv().await {
            let delta = item?;
            if let Some(text) = delta.text {
                deltas.push(text);
            }
        }
        assert_eq!(deltas, vec!["Yo", "lo"]);

        let request = request.recv()?;
        assert!(request.contains("\"system\":\"be brief\""));
        assert!(request.contains("\"role\":\"user\""));
        assert!(!request.contains("\"role\":\"system\""));
        Ok(())
    }
}
