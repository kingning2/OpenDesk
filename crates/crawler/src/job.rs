//! 采集任务状态、日志与 UI 事件。

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use common::contracts::{
    CrawlerEventChannelAccepted, CrawlerEventJobCompleted, CrawlerEventJobFailed,
    CrawlerEventJobLog, CrawlerEventJobProgress, CrawlerEventJobStarted,
    CrawlerIpcJobStatusResponse,
};
use ports::crawler_channels::ChannelRecord;
use serde_json::Value;
use uuid::Uuid;

use crate::i18n as msg;
use crate::{CrawlerUiEmitter, Locale};

#[derive(Debug, Clone)]
struct KeywordProgress {
    keyword: String,
    scanned: i64,
    accepted: i64,
}

#[derive(Debug, Clone)]
struct JobSnapshot {
    job_id: String,
    platform: String,
    status: String,
    stop_reason: Option<String>,
    message: Option<String>,
    current_keyword: Option<String>,
    scanned_count: i64,
    accepted_count: i64,
    keyword_scanned: i64,
    keyword_accepted: i64,
    quota_used: i64,
    keyword_stats: Vec<KeywordProgress>,
    keywords_total: i64,
    keywords_done: i64,
    error_message: Option<String>,
}

pub(super) struct JobHandle {
    snapshot: Mutex<JobSnapshot>,
    logs: Mutex<Vec<CrawlerEventJobLog>>,
    pub(super) cancel_requested: AtomicBool,
    seq: AtomicI64,
    locale: Locale,
    emitter: Arc<dyn CrawlerUiEmitter>,
}

impl JobHandle {
    pub(super) fn new(
        job_id: String,
        platform: String,
        locale: Locale,
        emitter: Arc<dyn CrawlerUiEmitter>,
    ) -> Self {
        Self {
            snapshot: Mutex::new(JobSnapshot {
                job_id,
                platform,
                status: "queued".to_string(),
                stop_reason: None,
                message: Some(msg::queued(locale)),
                current_keyword: None,
                scanned_count: 0,
                accepted_count: 0,
                keyword_scanned: 0,
                keyword_accepted: 0,
                quota_used: 0,
                keyword_stats: Vec::new(),
                keywords_total: 0,
                keywords_done: 0,
                error_message: None,
            }),
            logs: Mutex::new(Vec::new()),
            cancel_requested: AtomicBool::new(false),
            seq: AtomicI64::new(0),
            locale,
            emitter,
        }
    }

    pub(super) fn emit_started(&self, keywords: &[String]) {
        let (job_id, platform) = match self.snapshot.lock() {
            Ok(snapshot) => (snapshot.job_id.clone(), snapshot.platform.clone()),
            Err(_) => return,
        };
        self.emitter.emit_job_started(&CrawlerEventJobStarted {
            event_id: Uuid::new_v4().to_string(),
            occurred_at: now_string(),
            job_id,
            platform,
            keywords: Some(keywords.join(",")),
        });
        self.emit_progress_snapshot();
    }

    fn emit_progress_snapshot(&self) {
        let Ok(snapshot) = self.snapshot.lock() else {
            return;
        };
        self.emitter.emit_job_progress(&CrawlerEventJobProgress {
            event_id: Uuid::new_v4().to_string(),
            occurred_at: now_string(),
            job_id: snapshot.job_id.clone(),
            platform: snapshot.platform.clone(),
            status: Some(snapshot.status.clone()),
            message: snapshot.message.clone(),
            stop_reason: snapshot.stop_reason.clone(),
            current_keyword: snapshot.current_keyword.clone(),
            scanned_count: snapshot.scanned_count,
            accepted_count: snapshot.accepted_count,
            quota_used: Some(snapshot.quota_used),
            search_pages: None,
            keyword_scanned: Some(snapshot.keyword_scanned),
            keyword_accepted: Some(snapshot.keyword_accepted),
            keywords_total: Some(snapshot.keywords_total),
            keywords_done: Some(snapshot.keywords_done),
            keyword_stats_json: Some(keyword_stats_json(&snapshot.keyword_stats)),
            error_message: snapshot.error_message.clone(),
        });
    }

    pub(super) fn set_running(&self, keywords_total: usize) {
        if let Ok(mut snapshot) = self.snapshot.lock() {
            snapshot.status = "running".to_string();
            snapshot.keywords_total = keywords_total as i64;
            snapshot.keywords_done = 0;
            snapshot.message = Some(msg::prepare_keywords(self.locale, keywords_total));
        }
        self.emit_progress_snapshot();
    }

    pub(super) fn set_cancel_requested(&self) {
        if let Ok(mut snapshot) = self.snapshot.lock() {
            if snapshot.status == "queued" || snapshot.status == "running" {
                snapshot.status = "cancelled".to_string();
                snapshot.stop_reason = Some("cancelled".to_string());
                snapshot.message = Some(msg::cancelled(self.locale));
            }
        }
        self.emit_progress_snapshot();
    }

    pub(super) fn set_progress(
        &self,
        keyword: &str,
        keyword_scanned: i64,
        keyword_accepted: i64,
        scanned_count: i64,
        accepted_count: i64,
        quota_used: i64,
    ) {
        if let Ok(mut snapshot) = self.snapshot.lock() {
            snapshot.current_keyword = Some(keyword.to_string());
            snapshot.scanned_count = scanned_count;
            snapshot.accepted_count = accepted_count;
            snapshot.keyword_scanned = keyword_scanned;
            snapshot.keyword_accepted = keyword_accepted;
            snapshot.quota_used = quota_used;
            let done = (snapshot.keywords_done + 1).min(snapshot.keywords_total.max(1));
            let total = snapshot.keywords_total.max(1);
            snapshot.message = Some(msg::progress(
                self.locale,
                done,
                total,
                keyword,
                keyword_accepted,
                accepted_count,
            ));
            upsert_keyword_progress(
                &mut snapshot.keyword_stats,
                keyword,
                keyword_scanned,
                keyword_accepted,
            );
        }
        self.emit_progress_snapshot();
    }

    pub(super) fn set_completed(
        &self,
        stop_reason: &str,
        quota_used: i64,
        scanned_count: i64,
        accepted_count: i64,
        duration_ms: i64,
    ) {
        let (job_id, platform) = {
            if let Ok(mut snapshot) = self.snapshot.lock() {
                snapshot.status = "completed".to_string();
                snapshot.stop_reason = Some(stop_reason.to_string());
                snapshot.quota_used = quota_used;
                snapshot.scanned_count = scanned_count;
                snapshot.accepted_count = accepted_count;
                snapshot.message = Some(msg::stop_message(self.locale, stop_reason));
                (snapshot.job_id.clone(), snapshot.platform.clone())
            } else {
                return;
            }
        };
        self.emit_progress_snapshot();
        self.emitter.emit_job_completed(&CrawlerEventJobCompleted {
            event_id: Uuid::new_v4().to_string(),
            occurred_at: now_string(),
            job_id,
            platform,
            stop_reason: stop_reason.to_string(),
            scanned_count,
            accepted_count,
            quota_used: Some(quota_used),
            duration_ms: Some(duration_ms),
        });
    }

    pub(super) fn set_cancelled(&self) {
        if let Ok(mut snapshot) = self.snapshot.lock() {
            snapshot.status = "cancelled".to_string();
            snapshot.stop_reason = Some("cancelled".to_string());
            snapshot.message = Some(msg::cancelled(self.locale));
        }
        self.emit_progress_snapshot();
        let (job_id, platform, scanned, accepted, quota) = match self.snapshot.lock() {
            Ok(snapshot) => (
                snapshot.job_id.clone(),
                snapshot.platform.clone(),
                snapshot.scanned_count,
                snapshot.accepted_count,
                snapshot.quota_used,
            ),
            Err(_) => return,
        };
        self.emitter.emit_job_completed(&CrawlerEventJobCompleted {
            event_id: Uuid::new_v4().to_string(),
            occurred_at: now_string(),
            job_id,
            platform,
            stop_reason: "cancelled".to_string(),
            scanned_count: scanned,
            accepted_count: accepted,
            quota_used: Some(quota),
            duration_ms: None,
        });
    }

    pub(super) fn mark_keyword_done(&self) {
        if let Ok(mut snapshot) = self.snapshot.lock() {
            snapshot.keywords_done += 1;
            let done = snapshot.keywords_done;
            let total = snapshot.keywords_total;
            snapshot.message = Some(msg::keyword_done(self.locale, done, total));
        }
        self.emit_progress_snapshot();
    }

    pub(super) fn set_failed(&self, message: String) {
        let (job_id, platform) = {
            if let Ok(mut snapshot) = self.snapshot.lock() {
                snapshot.status = "failed".to_string();
                snapshot.stop_reason = Some("failed".to_string());
                snapshot.error_message = Some(message.clone());
                snapshot.message = Some(msg::failed(self.locale, &message));
                (snapshot.job_id.clone(), snapshot.platform.clone())
            } else {
                return;
            }
        };
        self.emit_progress_snapshot();
        self.emitter.emit_job_failed(&CrawlerEventJobFailed {
            event_id: Uuid::new_v4().to_string(),
            occurred_at: now_string(),
            job_id,
            platform,
            error_code: "crawl_failed".to_string(),
            message,
        });
    }

    pub(super) fn push_log(
        &self,
        platform: &str,
        phase: &str,
        message: String,
        keyword: Option<String>,
        detail: Option<String>,
    ) {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst) + 1;
        let job_id = self
            .snapshot
            .lock()
            .ok()
            .map(|snapshot| snapshot.job_id.clone())
            .unwrap_or_default();
        let log = CrawlerEventJobLog {
            event_id: Uuid::new_v4().to_string(),
            occurred_at: now_string(),
            job_id,
            platform: platform.to_string(),
            seq,
            phase: phase.to_string(),
            level: "INFO".to_string(),
            message,
            keyword,
            detail,
        };
        if let Ok(mut logs) = self.logs.lock() {
            logs.push(log.clone());
        }
        self.emitter.emit_job_log(&log);
        tracing::info!(
            target: "crawler",
            job_id = %log.job_id,
            phase = %log.phase,
            keyword = log.keyword.as_deref().unwrap_or("-"),
            "{}",
            log.message
        );
    }

    pub(super) fn status_response(&self) -> CrawlerIpcJobStatusResponse {
        let snapshot = self
            .snapshot
            .lock()
            .ok()
            .map(|value| value.clone())
            .unwrap_or(JobSnapshot {
                job_id: String::new(),
                platform: "youtube".to_string(),
                status: "failed".to_string(),
                stop_reason: Some("failed".to_string()),
                message: Some(msg::status_unavailable(self.locale)),
                current_keyword: None,
                scanned_count: 0,
                accepted_count: 0,
                keyword_scanned: 0,
                keyword_accepted: 0,
                quota_used: 0,
                keyword_stats: Vec::new(),
                keywords_total: 0,
                keywords_done: 0,
                error_message: Some("status lock poisoned".to_string()),
            });
        CrawlerIpcJobStatusResponse {
            ok: true,
            job_id: snapshot.job_id,
            platform: snapshot.platform,
            status: snapshot.status,
            stop_reason: snapshot.stop_reason,
            message: snapshot.message,
            current_keyword: snapshot.current_keyword,
            scanned_count: Some(snapshot.scanned_count),
            accepted_count: Some(snapshot.accepted_count),
            keyword_scanned: Some(snapshot.keyword_scanned),
            keyword_accepted: Some(snapshot.keyword_accepted),
            quota_used: Some(snapshot.quota_used),
            keywords_total: Some(snapshot.keywords_total),
            keywords_done: Some(snapshot.keywords_done),
            keyword_stats_json: Some(keyword_stats_json(&snapshot.keyword_stats)),
            error_message: snapshot.error_message,
            trace_id: None,
        }
    }

    pub(super) fn logs_json(&self) -> Result<String, String> {
        let logs = self.logs.lock().map_err(|error| error.to_string())?;
        serde_json::to_string(&*logs).map_err(|error| error.to_string())
    }

    pub(super) fn emit_channel_accepted(&self, record: &ChannelRecord) {
        self.emitter
            .emit_channel_accepted(&CrawlerEventChannelAccepted {
                event_id: Uuid::new_v4().to_string(),
                occurred_at: now_string(),
                job_id: record.job_id.clone(),
                platform: record.platform.clone(),
                keyword: record.keyword.clone(),
                channel_id: record.channel_id.clone(),
                title: record.title.clone(),
                country: record.country.clone(),
                subscriber_count: record.subscriber_count,
                email: record.email.clone(),
                description: record.description.clone(),
                custom_url: record.custom_url.clone(),
                email_status: record.email_status.clone(),
                enrich_attempts: Some(record.enrich_attempts as i64),
                enrich_error: record.enrich_error.clone(),
                enriched_at: record.enriched_at.clone(),
            });
    }
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn upsert_keyword_progress(
    rows: &mut Vec<KeywordProgress>,
    keyword: &str,
    scanned: i64,
    accepted: i64,
) {
    if let Some(row) = rows.iter_mut().find(|row| row.keyword == keyword) {
        row.scanned = scanned;
        row.accepted = accepted;
        return;
    }
    rows.push(KeywordProgress {
        keyword: keyword.to_string(),
        scanned,
        accepted,
    });
}

fn keyword_stats_json(rows: &[KeywordProgress]) -> String {
    let payload: Vec<Value> = rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "keyword": row.keyword,
                "scanned": row.scanned,
                "accepted": row.accepted,
            })
        })
        .collect();
    serde_json::to_string(&payload).unwrap_or_else(|_| "[]".to_string())
}
