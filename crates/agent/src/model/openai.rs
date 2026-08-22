//! OpenAI 兼容 provider — reqwest 直连 `POST /chat/completions`。
//!
//! 覆盖 OpenAI / DeepSeek / 豆包 Ark / 阿里云百炼兼容模式 / Ollama 等所有
//! `POST /chat/completions` 兼容服务。模型基址按兼容协议标准化。
//!
//! 推理模型（DeepSeek R1 / 豆包 Seed 等）默认思考会把 token 预算耗尽，
//! 导致 `content` 为空；调用方显式要求关闭思考时下发 `thinking` 字段。

use super::{ChatRequest, ChatResponse, LlmError, LlmProvider, ProviderSettings};
use async_trait::async_trait;
use serde_json::{json, Value};

/// 标准 OpenAI 兼容端点路径。
const DEFAULT_CHAT_PATH: &str = "/chat/completions";

/// 把基址规范化为 `api_base`（不含 `/chat/completions`，请求时自行拼接）。
fn normalize_base_url(base_url: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return "https://api.openai.com/v1".to_string();
    }
    // 去掉用户已填的 `/chat/completions` 结尾。
    let base = base.strip_suffix(DEFAULT_CHAT_PATH).unwrap_or(base);
    // 已含 /v1、/v2、/v3 则直接用。
    if base.ends_with("/v1") || base.ends_with("/v2") || base.ends_with("/v3") {
        return base.to_string();
    }
    format!("{base}/v1")
}

/// OpenAI 兼容 provider。
pub struct OpenAiCompatibleProvider {
    settings: ProviderSettings,
}

impl OpenAiCompatibleProvider {
    pub fn new(settings: ProviderSettings) -> Self {
        Self { settings }
    }
}

/// 构建 `/chat/completions` 请求体。
///
/// - 官方 OpenAI 用 `max_completion_tokens`；DeepSeek / 豆包 / Ollama 等用 `max_tokens`。
/// - `disable_thinking` 时下发 `thinking: {"type": "disabled"}`，避免推理模型 content 为空。
fn build_payload(request: &ChatRequest, is_openai_official: bool) -> Value {
    let messages: Vec<Value> = request
        .messages
        .iter()
        .map(|msg| json!({ "role": msg.role, "content": msg.content }))
        .collect();
    let mut payload = json!({
        "model": request.model,
        "messages": messages,
        "temperature": request.temperature,
    });
    if is_openai_official {
        payload["max_completion_tokens"] = json!(request.max_tokens);
    } else {
        payload["max_tokens"] = json!(request.max_tokens);
    }
    if request.disable_thinking {
        payload["thinking"] = json!({ "type": "disabled" });
    }
    payload
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn kind(&self) -> &'static str {
        "openai_compatible"
    }

    async fn complete(&self, request: &ChatRequest) -> Result<ChatResponse, LlmError> {
        let base_url = normalize_base_url(&self.settings.base_url);
        let is_openai_official = base_url.contains("api.openai.com");
        let url = format!("{base_url}{DEFAULT_CHAT_PATH}");
        let payload = build_payload(request, is_openai_official);

        let response = reqwest::Client::new()
            .post(&url)
            .bearer_auth(&self.settings.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|error| LlmError::Transport(format!("openai compatible: {error}")))?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::Provider(format!(
                "openai compatible: HTTP {status}: {text}"
            )));
        }
        let body: Value = response
            .json()
            .await
            .map_err(|error| LlmError::Transport(format!("openai compatible parse: {error}")))?;
        let choice = body
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .ok_or(LlmError::EmptyResponse)?;
        let message = choice.get("message").ok_or(LlmError::EmptyResponse)?;
        let reply = message
            .get("content")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|content| !content.is_empty())
            .map(str::to_string);
        let Some(reply) = reply else {
            let thinking = message
                .get("reasoning_content")
                .and_then(Value::as_str)
                .is_some_and(|content| !content.is_empty());
            return Err(if thinking {
                LlmError::Provider(
                    "模型处于思考模式且未输出正文（content 为空），请关闭思考或换用非推理模型"
                        .to_string(),
                )
            } else {
                LlmError::EmptyResponse
            });
        };
        let finish_reason = choice
            .get("finish_reason")
            .and_then(Value::as_str)
            .map(str::to_string);
        Ok(ChatResponse {
            reply,
            finish_reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ChatMessage;

    fn request(disable_thinking: bool) -> ChatRequest {
        ChatRequest {
            model: "doubao-seed-2-0-mini".to_string(),
            messages: vec![ChatMessage::user("hi")],
            max_tokens: 512,
            temperature: 0.2,
            disable_thinking,
        }
    }

    #[test]
    fn normalizes_common_base_urls() {
        assert_eq!(
            normalize_base_url("https://api.deepseek.com"),
            "https://api.deepseek.com/v1"
        );
        assert_eq!(
            normalize_base_url("https://dashscope.aliyuncs.com/compatible-mode/v1"),
            "https://dashscope.aliyuncs.com/compatible-mode/v1"
        );
        assert_eq!(
            normalize_base_url("https://ark.cn-beijing.volces.com/api/v3"),
            "https://ark.cn-beijing.volces.com/api/v3"
        );
        assert_eq!(normalize_base_url(""), "https://api.openai.com/v1");
    }

    #[test]
    fn strips_chat_path_from_full_endpoint() {
        assert_eq!(
            normalize_base_url("https://ark.cn-beijing.volces.com/api/v3/chat/completions"),
            "https://ark.cn-beijing.volces.com/api/v3"
        );
        assert_eq!(
            normalize_base_url("http://localhost:11434/v1/chat/completions"),
            "http://localhost:11434/v1"
        );
    }

    #[test]
    fn disables_thinking_when_requested() {
        let payload = build_payload(&request(true), false);
        assert_eq!(payload["max_tokens"], 512);
        assert_eq!(payload["thinking"]["type"], "disabled");
    }

    #[test]
    fn official_openai_uses_max_completion_tokens() {
        let payload = build_payload(&request(false), true);
        assert!(payload.get("max_completion_tokens").is_some());
        assert!(payload.get("max_tokens").is_none());
        assert!(payload.get("thinking").is_none());
    }
}
