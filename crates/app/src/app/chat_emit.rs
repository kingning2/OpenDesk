//! Bridge [`chat::ChatUIEmitter`] to Tauri window events.
//!
//! 作者：coisini

use chat::{ChatUIEmitter, ChatUIEvent};
use common::contracts::{ChatEventToken, ChatEventTool};
use tauri::{AppHandle, Emitter};

/// Emit chat contract events to the React webview via Tauri.
pub struct TauriChatEmitter {
    app: AppHandle,
}

impl TauriChatEmitter {
    /// Create an emitter bound to the running Tauri app.
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// Emit a typed payload on a [`ChatUIEvent`] topic.
    fn emit_payload<T: serde::Serialize>(&self, event: ChatUIEvent, payload: &T) {
        let topic = event.as_str();
        if let Err(error) = self.app.emit(topic, payload) {
            tracing::warn!(%topic, %error, "failed to emit chat UI event");
        }
    }
}

impl ChatUIEmitter for TauriChatEmitter {
    fn emit_message_token(&self, event: &ChatEventToken) {
        self.emit_payload(ChatUIEvent::MessageToken, event);
    }

    fn emit_message_tool(&self, event: &ChatEventTool) {
        self.emit_payload(ChatUIEvent::MessageTool, event);
    }
}
