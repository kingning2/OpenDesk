//! LLM provider 抽象 — 渠道无关的大模型接入。
//!
//! 统一 [`LlmProvider`] trait：任何 provider（OpenAI 兼容 / Anthropic / Gemini / DashScope）
//! 都实现 `complete(&ChatRequest) -> ChatResponse`。
//!
//! 第三方库优先：OpenAI 兼容走 `async-openai`；Anthropic / Gemini / DashScope App
//! 用 reqwest 薄封装（各自协议差异大，无统一第三方库）。

pub mod anthropic;
pub mod dashscope;
pub mod gemini;
pub mod openai;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// LLM 调用错误。
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("llm provider error: {0}")]
    Provider(String),
    #[error("llm transport error: {0}")]
    Transport(String),
    #[error("llm empty response")]
    EmptyResponse,
}

/// 对话消息（与契约 `LlmMessage` 对齐）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// 补全请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: u32,
    pub temperature: f32,
    /// 是否禁用思考（部分模型支持）。
    pub disable_thinking: bool,
}

/// 补全响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub reply: String,
    /// 截断原因（`length` 表示输出被截断，可重试）。
    pub finish_reason: Option<String>,
}

/// Provider 连接配置（从业务设置中提取）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    /// 规范化后的 provider 类型：openai_compatible / anthropic / gemini / dashscope_app。
    pub provider_type: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

/// LLM provider 统一接口。
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// provider 类型标识（openai_compatible / anthropic / gemini / dashscope_app）。
    fn kind(&self) -> &'static str;

    async fn complete(&self, request: &ChatRequest) -> Result<ChatResponse, LlmError>;
}

/// 按设置构造 provider（策略模式：类型 → 实现）。
pub fn provider_from_settings(
    settings: &ProviderSettings,
) -> Result<Box<dyn LlmProvider>, LlmError> {
    match settings.provider_type.as_str() {
        "anthropic" => Ok(Box::new(anthropic::AnthropicProvider::new(
            settings.clone(),
        ))),
        "gemini" => Ok(Box::new(gemini::GeminiProvider::new(settings.clone()))),
        "dashscope_app" => Ok(Box::new(dashscope::DashScopeAppProvider::new(
            settings.clone(),
        ))),
        // openai_compatible（含 dashscope 兼容模式 / ollama / deepseek 等）。
        _ => Ok(Box::new(openai::OpenAiCompatibleProvider::new(
            settings.clone(),
        ))),
    }
}

/// 规范化 provider 类型字符串（兼容旧配置无 provider 字段的场景）。
pub fn normalize_provider_type(provider_type: &str, base_url: &str, model: &str) -> String {
    let provider = provider_type.trim().to_lowercase().replace('-', "_");
    let provider = match provider.as_str() {
        "openai"
        | "openai_compatible"
        | "openai兼容"
        | "dashscope_compatible"
        | "qwen"
        | "dashscope" => "openai_compatible",
        "anthropic" | "claude" => "anthropic",
        "gemini" | "google_gemini" => "gemini",
        "dashscope_app" | "dashscope应用" => "dashscope_app",
        other => other,
    };
    if matches!(
        provider,
        "openai_compatible" | "anthropic" | "gemini" | "dashscope_app"
    ) {
        return provider.to_string();
    }

    let base = base_url.trim().to_lowercase();
    let model = model.trim().to_lowercase();
    if base.contains("generativelanguage.googleapis.com") {
        return "gemini".to_string();
    }
    if base.contains("api.anthropic.com") {
        return "anthropic".to_string();
    }
    if base.contains("/apps/") {
        return "dashscope_app".to_string();
    }
    if model.contains("gemini") {
        return "gemini".to_string();
    }
    if model.contains("claude") {
        return "anthropic".to_string();
    }
    "openai_compatible".to_string()
}

/// 去除首尾空白并清理换行（配置字段清洗）。
pub fn clean_text(value: &str) -> String {
    value.replace(['\r', '\n'], "").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_provider_types() {
        assert_eq!(
            normalize_provider_type("openai", "", ""),
            "openai_compatible"
        );
        assert_eq!(normalize_provider_type("qwen", "", ""), "openai_compatible");
        assert_eq!(normalize_provider_type("claude", "", ""), "anthropic");
        assert_eq!(
            normalize_provider_type("", "https://generativelanguage.googleapis.com", ""),
            "gemini"
        );
        assert_eq!(
            normalize_provider_type("", "https://dashscope.aliyuncs.com/api/v1/apps/xx", ""),
            "dashscope_app"
        );
        assert_eq!(
            normalize_provider_type("", "", "deepseek-chat"),
            "openai_compatible"
        );
    }

    #[test]
    fn builds_provider_by_type() {
        let settings = ProviderSettings {
            provider_type: "openai_compatible".to_string(),
            api_key: String::from("k"),
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
        };
        let provider = provider_from_settings(&settings).expect("provider");
        assert_eq!(provider.kind(), "openai_compatible");
    }
}
