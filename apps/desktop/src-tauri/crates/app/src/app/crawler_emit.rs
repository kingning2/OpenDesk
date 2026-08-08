//! Bridge [`crawler::youtube::CrawlerUIEmitter`] to Tauri window events.
//!
//! 作者：coisini
//! 创建时间：2026-07-21

use common::contracts::{
    CrawlerEventChannelAccepted, CrawlerEventJobCompleted, CrawlerEventJobFailed,
    CrawlerEventJobLog, CrawlerEventJobProgress, CrawlerEventJobStarted,
};
use crawler::youtube::{CrawlerUIEmitter, CrawlerUIEvent};
use tauri::{AppHandle, Emitter};

/// Emit crawler contract events to the React webview via Tauri.
///
/// 作者：coisini
/// 创建时间：2026-07-21
pub struct TauriCrawlerEmitter {
    app: AppHandle,
}

impl TauriCrawlerEmitter {
    /// Create an emitter bound to the running Tauri app.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-21
    ///
    /// # 参数
    /// - `app` — Tauri app handle used for `emit`
    ///
    /// # 返回值
    /// Emitter ready to attach to [`crawler::youtube::CrawlerService`].
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// Emit a typed payload on a [`CrawlerUIEvent`] topic.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-21
    ///
    /// # 参数
    /// - `event` — topic enum（禁止散落字符串）
    /// - `payload` — contract event DTO
    fn emit_payload<T: serde::Serialize>(&self, event: CrawlerUIEvent, payload: &T) {
        let topic = event.as_str();
        if let Err(error) = self.app.emit(topic, payload) {
            tracing::warn!(%topic, %error, "failed to emit crawler UI event");
        }
    }
}

impl CrawlerUIEmitter for TauriCrawlerEmitter {
    fn emit_job_started(&self, event: &CrawlerEventJobStarted) {
        self.emit_payload(CrawlerUIEvent::JobStarted, event);
    }

    fn emit_job_progress(&self, event: &CrawlerEventJobProgress) {
        self.emit_payload(CrawlerUIEvent::JobProgress, event);
    }

    fn emit_job_log(&self, event: &CrawlerEventJobLog) {
        self.emit_payload(CrawlerUIEvent::JobLog, event);
    }

    fn emit_job_completed(&self, event: &CrawlerEventJobCompleted) {
        self.emit_payload(CrawlerUIEvent::JobCompleted, event);
    }

    fn emit_job_failed(&self, event: &CrawlerEventJobFailed) {
        self.emit_payload(CrawlerUIEvent::JobFailed, event);
    }

    fn emit_channel_accepted(&self, event: &CrawlerEventChannelAccepted) {
        self.emit_payload(CrawlerUIEvent::ChannelAccepted, event);
    }
}
