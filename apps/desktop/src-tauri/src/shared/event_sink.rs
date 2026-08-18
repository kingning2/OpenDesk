//! Tauri `EventSink` 适配器 — 将 `AppEvent` 序列化后通过 `tauri::Emitter` 推送至前端。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use common::events::EventSink;
use common::OpenDeskResult;
use tauri::{AppHandle, Emitter};

/// 将 `common::events::EventSink` 桥接到 `tauri::AppHandle::emit`。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
pub struct TauriEventSink {
    app: AppHandle,
}

impl TauriEventSink {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl EventSink for TauriEventSink {
    fn publish(&self, topic: &str, payload: &[u8]) -> OpenDeskResult<()> {
        let json: serde_json::Value = serde_json::from_slice(payload)
            .map_err(|e| common::OpenDeskError::Serialization(e.to_string()))?;
        self.app
            .emit(topic, json)
            .map_err(|e| common::OpenDeskError::Internal(e.to_string()))
    }
}
