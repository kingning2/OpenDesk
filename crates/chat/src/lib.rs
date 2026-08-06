//! 桌面端 AI 聊天能力：把对话转发给 LLM 并流式回传 token。
//!
//! 本 crate **不持有 Tauri AppHandle**；事件通过 [`ChatUiEmitter`] 抽象推送，
//! 真正的 Tauri 实现放在 `crates/app`（对照 crawler 的 `TauriCrawlerEmitter`）。
//!
//! 约定：一个能力一个目录（`emit/`、`app/` …）。

pub mod app;
pub mod emit;
pub mod tool;

pub use app::memory_digest::maybe_digest;
pub use app::send_chat::SendChat;
pub use emit::{ChatUiEmitter, ChatUiEvent, NoopChatUiEmitter};
pub use tool::{ChatTool, ChatToolCaller, CompositeChatToolCaller};
