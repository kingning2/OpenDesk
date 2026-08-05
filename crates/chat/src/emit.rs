//! Chat UI event sink — Tauri / noop implementations live outside this crate.
//!
//! Topic names come from [`ChatUiEvent`] (`chat:<entity>/<verb>`).
//! Tauri only allows alphanumeric, `-`, `/`, `:`, `_` (no `.`).

use std::fmt;

use common::contracts::{ChatEventToken, ChatEventTool};

/// Chat → UI Tauri event topics (single source of truth for topic strings).
///
/// 作者：coisini
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChatUiEvent {
    /// One streamed token of an assistant reply (delta or final done).
    MessageToken,
    /// One tool call executed by the assistant during a reply.
    MessageTool,
}

impl ChatUiEvent {
    /// Tauri event name string (safe charset only).
    ///
    /// # 返回值
    /// Topic such as `chat:message/token`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MessageToken => "chat:message/token",
            Self::MessageTool => "chat:message/tool",
        }
    }
}

impl AsRef<str> for ChatUiEvent {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ChatUiEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Push chat streaming events to the desktop UI.
///
/// 实现方：
/// - [`NoopChatUiEmitter`] — 测试 / 未接线时丢弃
/// - `crates/app` 中的 Tauri `Emitter` 适配器
pub trait ChatUiEmitter: Send + Sync {
    /// Emit one streamed token (or the final done event) of an assistant reply.
    fn emit_message_token(&self, event: &ChatEventToken);

    /// Emit one tool call executed during a reply.
    fn emit_message_tool(&self, event: &ChatEventTool);
}

/// Drop-all emitter used until the Tauri app handle is attached.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopChatUiEmitter;

impl ChatUiEmitter for NoopChatUiEmitter {
    fn emit_message_token(&self, _event: &ChatEventToken) {}

    fn emit_message_tool(&self, _event: &ChatEventTool) {}
}

#[cfg(test)]
mod tests {
    use super::ChatUiEvent;

    #[test]
    fn topic_strings_have_no_dot() {
        let events = [ChatUiEvent::MessageToken, ChatUiEvent::MessageTool];
        for event in events {
            assert!(
                !event.as_str().contains('.'),
                "{} must not contain '.'",
                event.as_str()
            );
        }
    }
}
