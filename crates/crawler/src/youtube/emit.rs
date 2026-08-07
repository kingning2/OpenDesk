//! Crawl UI event sink — Tauri / noop implementations live outside this crate.
//!
//! Topic names come from [`CrawlerUIEvent`] (`crawler:<entity>/<verb>`).
//! Tauri only allows alphanumeric, `-`, `/`, `:`, `_` (no `.`).
//!
//! 作者：coisini
//! 创建时间：2026-07-21

use common::contracts::{
    CrawlerEventChannelAccepted, CrawlerEventJobCompleted, CrawlerEventJobFailed,
    CrawlerEventJobLog, CrawlerEventJobProgress, CrawlerEventJobStarted,
};
use std::fmt;

/// Crawler → UI Tauri event topics (single source of truth for topic strings).
///
/// 作者：coisini
/// 创建时间：2026-07-21
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrawlerUIEvent {
    /// Job entered running / keywords prepared.
    JobStarted,
    /// Progress snapshot (counts, message, keyword stats).
    JobProgress,
    /// One process log line.
    JobLog,
    /// Job finished successfully or cancelled.
    JobCompleted,
    /// Job failed with error.
    JobFailed,
    /// One accepted channel row persisted.
    ChannelAccepted,
    /// Email enrichment finished for one channel.
    ChannelEmailEnriched,
}

impl CrawlerUIEvent {
    /// Tauri event name string (safe charset only).
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-21
    ///
    /// # 返回值
    /// Topic such as `crawler:job/progress`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::JobStarted => "crawler:job/started",
            Self::JobProgress => "crawler:job/progress",
            Self::JobLog => "crawler:job/log",
            Self::JobCompleted => "crawler:job/completed",
            Self::JobFailed => "crawler:job/failed",
            Self::ChannelAccepted => "crawler:channel/accepted",
            Self::ChannelEmailEnriched => "crawler:channel/email_enriched",
        }
    }
}

impl AsRef<str> for CrawlerUIEvent {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for CrawlerUIEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Push crawl lifecycle events to the desktop UI (replaces status/logs/results polling).
///
/// 实现方：
/// - [`NoopCrawlerUIEmitter`] — 测试 / 未接线时丢弃
/// - `crates/app` 中的 Tauri `Emitter` 适配器
///
/// 作者：coisini
/// 创建时间：2026-07-21
pub trait CrawlerUIEmitter: Send + Sync {
    /// Emit [`CrawlerUIEvent::JobStarted`].
    fn emit_job_started(&self, event: &CrawlerEventJobStarted);

    /// Emit [`CrawlerUIEvent::JobProgress`].
    fn emit_job_progress(&self, event: &CrawlerEventJobProgress);

    /// Emit [`CrawlerUIEvent::JobLog`].
    fn emit_job_log(&self, event: &CrawlerEventJobLog);

    /// Emit [`CrawlerUIEvent::JobCompleted`].
    fn emit_job_completed(&self, event: &CrawlerEventJobCompleted);

    /// Emit [`CrawlerUIEvent::JobFailed`].
    fn emit_job_failed(&self, event: &CrawlerEventJobFailed);

    /// Emit [`CrawlerUIEvent::ChannelAccepted`].
    fn emit_channel_accepted(&self, event: &CrawlerEventChannelAccepted);
}

/// Drop-all emitter used until the Tauri app handle is attached.
///
/// 作者：coisini
/// 创建时间：2026-07-21
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopCrawlerUIEmitter;

impl CrawlerUIEmitter for NoopCrawlerUIEmitter {
    fn emit_job_started(&self, _event: &CrawlerEventJobStarted) {}
    fn emit_job_progress(&self, _event: &CrawlerEventJobProgress) {}
    fn emit_job_log(&self, _event: &CrawlerEventJobLog) {}
    fn emit_job_completed(&self, _event: &CrawlerEventJobCompleted) {}
    fn emit_job_failed(&self, _event: &CrawlerEventJobFailed) {}
    fn emit_channel_accepted(&self, _event: &CrawlerEventChannelAccepted) {}
}

#[cfg(test)]
mod tests {
    use super::CrawlerUIEvent;

    #[test]
    fn topic_strings_have_no_dot() {
        for event in [
            CrawlerUIEvent::JobStarted,
            CrawlerUIEvent::JobProgress,
            CrawlerUIEvent::JobLog,
            CrawlerUIEvent::JobCompleted,
            CrawlerUIEvent::JobFailed,
            CrawlerUIEvent::ChannelAccepted,
            CrawlerUIEvent::ChannelEmailEnriched,
        ] {
            assert!(
                !event.as_str().contains('.'),
                "{} must not contain '.'",
                event.as_str()
            );
        }
    }
}
