//! agent crate — AI 基建（渠道无关）。
//!
//! 目录职责：
//! - `llm` — LLM provider 抽象 + 多协议接入（OpenAI 兼容 / Anthropic / Gemini / DashScope）
//! - `intent` — 本地意图检测（price / tech / default / no_reply）
//! - `prompt` — 提示词模板与组装（议价 / 技术 / 默认客服）
//! - `knowledge` — 知识库（商品信息提取与注入）
//! - `reply` — 回复引擎（上下文管理、议价控制、provider 分发）
//!
//! 设计约束：
//! - **渠道无关**：不依赖 platform / app，可被任何业务（闲鱼、邮件、未来平台）复用。
//! - **不感知存储**：通过入参注入，不直接访问数据库。
//! - 边界情况从简：能 `map` 就不写 `if`；错误一律 `Result` + `tracing`，禁止 `unwrap` 崩。

pub mod intent;
pub mod knowledge;
pub mod llm;
pub mod prompt;
pub mod reply;

pub use intent::{route_intent, Intent};
pub use knowledge::{build_item_context, ItemKnowledge};
pub use llm::{
    provider_from_settings, ChatMessage, ChatRequest, ChatResponse, LlmError, LlmProvider,
    ProviderSettings,
};
pub use prompt::PromptBuilder;
pub use reply::{AiSettings, ReplyContext, ReplyEngine, ReplyOutcome};
