//! OpenAI 兼容 provider — 基于 `async-openai` 第三方库。
//!
//! 覆盖 OpenAI / DeepSeek / 阿里云百炼兼容模式 / Ollama / Kimi 等所有
//! `POST /chat/completions` 兼容服务。模型基址按兼容协议标准化。

use super::{ChatMessage, ChatRequest, ChatResponse, LlmError, LlmProvider, ProviderSettings};
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequest,
};
use async_openai::Client;
use async_trait::async_trait;

/// 标准 OpenAI 兼容端点（缺省时由基址推导）。
const DEFAULT_CHAT_PATH: &str = "/chat/completions";

/// 把基址规范化为指向 `/chat/completions` 的完整 URL。
fn normalize_base_url(base_url: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return format!("https://api.openai.com/v1{DEFAULT_CHAT_PATH}");
    }
    if base.ends_with("/chat/completions") || base.contains("/chat/completions") {
        return base.to_string();
    }
    // 已含 /v1 或 /v1/ 则直接拼路径。
    if base.ends_with("/v1") {
        return format!("{base}{DEFAULT_CHAT_PATH}");
    }
    format!("{base}/v1{DEFAULT_CHAT_PATH}")
}

/// OpenAI 兼容 provider。
pub struct OpenAiCompatibleProvider {
    settings: ProviderSettings,
}

impl OpenAiCompatibleProvider {
    pub fn new(settings: ProviderSettings) -> Self {
        Self { settings }
    }

    fn build_request(
        &self,
        request: &ChatRequest,
    ) -> Result<CreateChatCompletionRequest, LlmError> {
        let messages = request
            .messages
            .iter()
            .map(to_openai_message)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CreateChatCompletionRequest {
            model: request.model.clone(),
            messages,
            max_completion_tokens: Some(request.max_tokens),
            temperature: Some(request.temperature),
            ..Default::default()
        })
    }
}

fn to_openai_message(msg: &ChatMessage) -> Result<ChatCompletionRequestMessage, LlmError> {
    match msg.role.as_str() {
        "system" => Ok(ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(msg.content.clone()),
                ..Default::default()
            },
        )),
        "assistant" => Ok(ChatCompletionRequestMessage::Assistant(
            ChatCompletionRequestAssistantMessage {
                content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                    msg.content.clone(),
                )),
                ..Default::default()
            },
        )),
        _ => Ok(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(msg.content.clone()),
                ..Default::default()
            },
        )),
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn kind(&self) -> &'static str {
        "openai_compatible"
    }

    async fn complete(&self, request: &ChatRequest) -> Result<ChatResponse, LlmError> {
        let base_url = normalize_base_url(&self.settings.base_url);
        let config = OpenAIConfig::new()
            .with_api_key(&self.settings.api_key)
            .with_api_base(base_url.clone());
        let client = Client::with_config(config);
        let api_request = self.build_request(request)?;

        let response = client
            .chat()
            .create(api_request)
            .await
            .map_err(|error| LlmError::Provider(format!("openai compatible: {error}")))?;

        let choice = response.choices.first().ok_or(LlmError::EmptyResponse)?;
        let reply = choice
            .message
            .content
            .as_ref()
            .map(|content| content.trim().to_string())
            .unwrap_or_default();
        if reply.is_empty() {
            return Err(LlmError::EmptyResponse);
        }

        let finish_reason = choice
            .finish_reason
            .as_ref()
            .map(|reason| format!("{reason:?}"))
            .map(|reason| reason.to_lowercase());
        Ok(ChatResponse {
            reply,
            finish_reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_common_base_urls() {
        assert_eq!(
            normalize_base_url("https://api.deepseek.com"),
            "https://api.deepseek.com/v1/chat/completions"
        );
        assert_eq!(
            normalize_base_url("https://dashscope.aliyuncs.com/compatible-mode/v1"),
            "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"
        );
        assert_eq!(
            normalize_base_url("http://localhost:11434/v1"),
            "http://localhost:11434/v1/chat/completions"
        );
        assert_eq!(
            normalize_base_url(""),
            "https://api.openai.com/v1/chat/completions"
        );
    }
}
