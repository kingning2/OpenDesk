//! Rust 直连 LLM 的最小 HTTP 客户端。

use std::time::Duration;

use reqwest::{Client as HttpClient, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

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

    use super::{Config, Error, LlmClient, Strategy};
    use crate::prompt::Prompt;

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
}
