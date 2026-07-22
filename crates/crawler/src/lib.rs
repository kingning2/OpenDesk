//! In-process YouTube crawler for the desktop app.
//!
//! Jobs run entirely in Rust: a supervisor thread fans out keyword work to a small
//! worker pool, calls the YouTube Data API with `reqwest`, and pushes UI updates
//! via [`CrawlerUiEmitter`] (Tauri Event). Email enrich enqueue is best-effort and
//! never fails the main crawl.
//!
//! 作者：coisini
//! 创建时间：2026-07-16

mod emit;
mod i18n;

pub use emit::{CrawlerUiEmitter, CrawlerUiEvent, NoopCrawlerUiEmitter};
pub use i18n::{empty_batch, need_batch, Locale};

use common::contracts::{
    CrawlerEventChannelAccepted, CrawlerEventJobCompleted, CrawlerEventJobFailed,
    CrawlerEventJobLog, CrawlerEventJobProgress, CrawlerEventJobStarted,
    CrawlerIpcJobCancelRequest, CrawlerIpcJobCancelResponse, CrawlerIpcJobLogsRequest,
    CrawlerIpcJobLogsResponse, CrawlerIpcJobStartRequest, CrawlerIpcJobStartResponse,
    CrawlerIpcJobStatusRequest, CrawlerIpcJobStatusResponse,
};
use i18n as msg;
use ports::background_job::{
    BackgroundJobStore, CrawlerEmailEnrichPayload, EMAIL_STATUS_PENDING_ENRICH,
    JOB_TYPE_CRAWLER_EMAIL_ENRICH,
};
use ports::crawler_channels::{ChannelRecord, CrawlerChannelStore};
use reqwest::blocking::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const API_BASE: &str = "https://www.googleapis.com/youtube/v3";
const USER_AGENT: &str = "OpenDeskCrawler/0.1";

/// Process-wide crawl orchestrator keyed by `job_id`.
///
/// 作者：coisini
/// 创建时间：2026-07-16
#[derive(Clone)]
pub struct CrawlerService {
    jobs: Arc<Mutex<HashMap<String, Arc<JobHandle>>>>,
    channels: Arc<dyn CrawlerChannelStore>,
    /// UI event sink (Tauri). Swappable after app setup.
    emitter: Arc<Mutex<Arc<dyn CrawlerUiEmitter>>>,
    /// Optional `opendesk.db` job queue for email enrich (never blocks crawl).
    jobs_queue: Arc<Mutex<Option<Arc<dyn BackgroundJobStore>>>>,
}

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

struct JobHandle {
    snapshot: Mutex<JobSnapshot>,
    logs: Mutex<Vec<CrawlerEventJobLog>>,
    cancel_requested: AtomicBool,
    seq: AtomicI64,
    /// Job 用户可见语言（启动时固定）。
    locale: Locale,
    emitter: Arc<dyn CrawlerUiEmitter>,
}

#[derive(Clone)]
struct RunConfig {
    job_id: String,
    platform: String,
    api_key: String,
    keywords: Vec<String>,
    rate_limit_ms: u64,
    max_total: i64,
    year: i32,
    min_year_video_count: i64,
    exclude_countries: Vec<String>,
    channels: Arc<dyn CrawlerChannelStore>,
    jobs_queue: Option<Arc<dyn BackgroundJobStore>>,
}

#[derive(Debug)]
enum CrawlError {
    QuotaExceeded,
    Cancelled,
    Message(String),
}

impl CrawlerService {
    /// Create a crawler service with a no-op UI emitter.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-16
    ///
    /// # 参数
    /// - `channels` — accepted-channel store (`crawler.db`)
    ///
    /// # 返回值
    /// Shared crawler service; call [`Self::attach_emitter`] after Tauri setup.
    pub fn new(channels: Arc<dyn CrawlerChannelStore>) -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            channels,
            emitter: Arc::new(Mutex::new(
                Arc::new(NoopCrawlerUiEmitter) as Arc<dyn CrawlerUiEmitter>
            )),
            jobs_queue: Arc::new(Mutex::new(None)),
        }
    }

    /// Attach a UI emitter (typically Tauri). Safe to call once during app setup.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-21
    ///
    /// # 参数
    /// - `emitter` — event sink used by subsequent jobs
    pub fn attach_emitter(&self, emitter: Arc<dyn CrawlerUiEmitter>) {
        if let Ok(mut slot) = self.emitter.lock() {
            *slot = emitter;
        }
    }

    /// Attach optional background job store for email enrich enqueue.
    ///
    /// Enqueue failures are logged only and never fail the crawl.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-21
    ///
    /// # 参数
    /// - `jobs_queue` — `opendesk.db` background job store
    pub fn attach_job_store(&self, jobs_queue: Arc<dyn BackgroundJobStore>) {
        if let Ok(mut slot) = self.jobs_queue.lock() {
            *slot = Some(jobs_queue);
        }
    }

    fn current_emitter(&self) -> Arc<dyn CrawlerUiEmitter> {
        self.emitter
            .lock()
            .map(|slot| slot.clone())
            .unwrap_or_else(|_| Arc::new(NoopCrawlerUiEmitter) as Arc<dyn CrawlerUiEmitter>)
    }

    fn current_job_store(&self) -> Option<Arc<dyn BackgroundJobStore>> {
        self.jobs_queue.lock().ok().and_then(|slot| slot.clone())
    }

    /// Start an in-process YouTube crawl job.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-16
    pub fn start(
        &self,
        request: CrawlerIpcJobStartRequest,
        keywords: Vec<String>,
    ) -> Result<CrawlerIpcJobStartResponse, String> {
        let platform = request.platform.trim().to_string();
        if platform != "youtube" {
            return Err(format!("unsupported crawler platform: {platform}"));
        }
        let api_key = request.api_key.trim().to_string();
        if api_key.is_empty() {
            return Err("api_key is required".to_string());
        }
        if keywords.is_empty() {
            return Err("keywords are required".to_string());
        }

        let job_id = Uuid::new_v4().to_string();
        let locale = Locale::parse(request.locale.as_deref());
        let emitter = self.current_emitter();
        let handle = Arc::new(JobHandle::new(
            job_id.clone(),
            platform.clone(),
            locale,
            emitter.clone(),
        ));
        self.jobs
            .lock()
            .map_err(|error| error.to_string())?
            .insert(job_id.clone(), handle.clone());

        let service = self.clone();
        let config = RunConfig {
            job_id: job_id.clone(),
            platform,
            api_key,
            keywords,
            rate_limit_ms: request.rate_limit_ms.unwrap_or(0).max(0) as u64,
            max_total: request.max_total.unwrap_or(0).max(0),
            year: request.year.unwrap_or(2025) as i32,
            min_year_video_count: request.min_year_video_count.unwrap_or(10).max(0),
            exclude_countries: split_csv(request.exclude_countries.as_deref()),
            channels: self.channels.clone(),
            jobs_queue: self.current_job_store(),
        };
        let trace_id = request.trace_id.clone();
        thread::Builder::new()
            .name(format!("crawler-supervisor-{}", &job_id[..8]))
            .spawn(move || service.run_job(handle, config))
            .map_err(|error| error.to_string())?;

        Ok(CrawlerIpcJobStartResponse {
            ok: true,
            job_id,
            trace_id,
        })
    }

    pub fn cancel(
        &self,
        request: CrawlerIpcJobCancelRequest,
    ) -> Result<CrawlerIpcJobCancelResponse, String> {
        let handle = self.job(&request.job_id)?;
        handle.cancel_requested.store(true, Ordering::SeqCst);
        handle.set_cancel_requested();
        Ok(CrawlerIpcJobCancelResponse {
            ok: true,
            job_id: request.job_id,
            trace_id: request.trace_id,
        })
    }

    pub fn status(
        &self,
        request: CrawlerIpcJobStatusRequest,
    ) -> Result<CrawlerIpcJobStatusResponse, String> {
        let handle = self.job(&request.job_id)?;
        let mut response = handle.status_response();
        response.trace_id = request.trace_id;
        Ok(response)
    }

    pub fn logs(
        &self,
        request: CrawlerIpcJobLogsRequest,
    ) -> Result<CrawlerIpcJobLogsResponse, String> {
        let handle = self.job(&request.job_id)?;
        let logs = handle.logs_json()?;
        Ok(CrawlerIpcJobLogsResponse {
            ok: true,
            job_id: request.job_id,
            logs_json: logs,
            trace_id: request.trace_id,
        })
    }

    fn job(&self, job_id: &str) -> Result<Arc<JobHandle>, String> {
        self.jobs
            .lock()
            .map_err(|error| error.to_string())?
            .get(job_id)
            .cloned()
            .ok_or_else(|| format!("unknown job_id={job_id}"))
    }

    /// Spawn up to four worker threads; each claims the next keyword index atomically.
    fn run_job(&self, handle: Arc<JobHandle>, config: RunConfig) {
        handle.set_running(config.keywords.len());
        handle.emit_started(&config.keywords);
        handle.push_log(
            &config.platform,
            "job_started",
            format!("youtube api job started keywords={}", config.keywords.len()),
            None,
            None,
        );

        let client = match Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .build()
        {
            Ok(client) => client,
            Err(error) => {
                handle.set_failed(format!("failed to create http client: {error}"));
                return;
            }
        };

        let next_index = Arc::new(AtomicUsize::new(0));
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_reason = Arc::new(Mutex::new(None::<String>));
        let scanned_total = Arc::new(AtomicI64::new(0));
        let accepted_total = Arc::new(AtomicI64::new(0));
        let search_pages = Arc::new(AtomicI64::new(0));
        let channel_calls = Arc::new(AtomicI64::new(0));
        let playlist_pages = Arc::new(AtomicI64::new(0));
        let started_at = SystemTime::now();

        let worker_count = config.keywords.len().clamp(1, 4);
        let mut workers = Vec::with_capacity(worker_count);

        for worker_idx in 0..worker_count {
            let client = client.clone();
            let worker_handle = handle.clone();
            let config = config.clone();
            let next_index = next_index.clone();
            let stop_flag = stop_flag.clone();
            let stop_reason = stop_reason.clone();
            let scanned_total = scanned_total.clone();
            let accepted_total = accepted_total.clone();
            let search_pages = search_pages.clone();
            let channel_calls = channel_calls.clone();
            let playlist_pages = playlist_pages.clone();

            let builder = thread::Builder::new().name(format!("crawler-worker-{worker_idx}"));
            let spawn_result = builder.spawn(move || loop {
                if stop_flag.load(Ordering::SeqCst)
                    || worker_handle.cancel_requested.load(Ordering::SeqCst)
                {
                    break Ok(()) as Result<(), CrawlError>;
                }
                if reached_max_total(&config, accepted_total.load(Ordering::SeqCst)) {
                    set_stop_reason(&stop_reason, "max_total_reached");
                    stop_flag.store(true, Ordering::SeqCst);
                    break Ok(());
                }

                let index = next_index.fetch_add(1, Ordering::SeqCst);
                let Some(keyword) = config.keywords.get(index).cloned() else {
                    break Ok(());
                };
                crawl_keyword(
                    &client,
                    &worker_handle,
                    &config,
                    &keyword,
                    &stop_flag,
                    &stop_reason,
                    &scanned_total,
                    &accepted_total,
                    &search_pages,
                    &channel_calls,
                    &playlist_pages,
                )?;
            });

            match spawn_result {
                Ok(worker) => workers.push(worker),
                Err(error) => {
                    handle.set_failed(format!("failed to spawn worker: {error}"));
                    return;
                }
            }
        }

        let mut failure: Option<CrawlError> = None;
        for worker in workers {
            match worker.join() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    if failure.is_none() {
                        failure = Some(error);
                    }
                }
                Err(_) => {
                    failure = Some(CrawlError::Message("crawler worker panicked".to_string()));
                }
            }
        }

        let duration_ms = started_at
            .elapsed()
            .map(|value| value.as_millis() as i64)
            .unwrap_or(0);
        let quota_used = calculate_expected_quota(
            search_pages.load(Ordering::SeqCst),
            channel_calls.load(Ordering::SeqCst),
            playlist_pages.load(Ordering::SeqCst),
        );

        match failure {
            Some(CrawlError::QuotaExceeded) => {
                handle.push_log(
                    &config.platform,
                    "quota",
                    "YouTube quotaExceeded - stopping".to_string(),
                    None,
                    None,
                );
                handle.set_completed(
                    "quota_exceeded",
                    quota_used,
                    scanned_total.load(Ordering::SeqCst),
                    accepted_total.load(Ordering::SeqCst),
                    duration_ms,
                );
            }
            Some(CrawlError::Cancelled) => {
                handle.set_cancelled();
            }
            Some(CrawlError::Message(message)) => {
                handle.set_failed(message);
            }
            None => {
                if handle.cancel_requested.load(Ordering::SeqCst) {
                    handle.set_cancelled();
                    return;
                }
                let reason = stop_reason
                    .lock()
                    .map(|value| {
                        value
                            .clone()
                            .unwrap_or_else(|| "keywords_finished".to_string())
                    })
                    .unwrap_or_else(|_| "keywords_finished".to_string());
                handle.set_completed(
                    &reason,
                    quota_used,
                    scanned_total.load(Ordering::SeqCst),
                    accepted_total.load(Ordering::SeqCst),
                    duration_ms,
                );
            }
        }
    }
}

impl JobHandle {
    fn new(
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

    fn emit_started(&self, keywords: &[String]) {
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

    fn set_running(&self, keywords_total: usize) {
        if let Ok(mut snapshot) = self.snapshot.lock() {
            snapshot.status = "running".to_string();
            snapshot.keywords_total = keywords_total as i64;
            snapshot.keywords_done = 0;
            snapshot.message = Some(msg::prepare_keywords(self.locale, keywords_total));
        }
        self.emit_progress_snapshot();
    }

    fn set_cancel_requested(&self) {
        if let Ok(mut snapshot) = self.snapshot.lock() {
            if snapshot.status == "queued" || snapshot.status == "running" {
                snapshot.status = "cancelled".to_string();
                snapshot.stop_reason = Some("cancelled".to_string());
                snapshot.message = Some(msg::cancelled(self.locale));
            }
        }
        self.emit_progress_snapshot();
    }

    fn set_progress(
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

    fn set_completed(
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

    fn set_cancelled(&self) {
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

    fn mark_keyword_done(&self) {
        if let Ok(mut snapshot) = self.snapshot.lock() {
            snapshot.keywords_done += 1;
            let done = snapshot.keywords_done;
            let total = snapshot.keywords_total;
            snapshot.message = Some(msg::keyword_done(self.locale, done, total));
        }
        self.emit_progress_snapshot();
    }

    fn set_failed(&self, message: String) {
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

    fn push_log(
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

    fn status_response(&self) -> CrawlerIpcJobStatusResponse {
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

    fn logs_json(&self) -> Result<String, String> {
        let logs = self.logs.lock().map_err(|error| error.to_string())?;
        serde_json::to_string(&*logs).map_err(|error| error.to_string())
    }
}

/// Crawl one keyword: search.list → channels.list → optional playlistItems filter.
#[allow(clippy::too_many_arguments)]
fn crawl_keyword(
    client: &Client,
    handle: &Arc<JobHandle>,
    config: &RunConfig,
    keyword: &str,
    stop_flag: &Arc<AtomicBool>,
    stop_reason: &Arc<Mutex<Option<String>>>,
    scanned_total: &Arc<AtomicI64>,
    accepted_total: &Arc<AtomicI64>,
    search_pages: &Arc<AtomicI64>,
    channel_calls: &Arc<AtomicI64>,
    playlist_pages: &Arc<AtomicI64>,
) -> Result<(), CrawlError> {
    if stop_flag.load(Ordering::SeqCst) || handle.cancel_requested.load(Ordering::SeqCst) {
        return Err(CrawlError::Cancelled);
    }

    handle.push_log(
        &config.platform,
        "keyword_begin",
        format!("begin keyword={keyword}"),
        Some(keyword.to_string()),
        None,
    );

    let mut keyword_scanned = 0i64;
    let mut keyword_accepted = 0i64;
    let mut page_token: Option<String> = None;
    handle.set_progress(
        keyword,
        keyword_scanned,
        keyword_accepted,
        scanned_total.load(Ordering::SeqCst),
        accepted_total.load(Ordering::SeqCst),
        calculate_expected_quota(
            search_pages.load(Ordering::SeqCst),
            channel_calls.load(Ordering::SeqCst),
            playlist_pages.load(Ordering::SeqCst),
        ),
    );

    loop {
        if stop_flag.load(Ordering::SeqCst) || handle.cancel_requested.load(Ordering::SeqCst) {
            return Err(CrawlError::Cancelled);
        }
        if reached_max_total(config, accepted_total.load(Ordering::SeqCst)) {
            set_stop_reason(stop_reason, "max_total_reached");
            stop_flag.store(true, Ordering::SeqCst);
            break;
        }

        sleep_rate(config.rate_limit_ms);
        search_pages.fetch_add(1, Ordering::SeqCst);
        let search_body = search_channels(client, config, keyword, page_token.as_deref())?;
        handle.push_log(
            &config.platform,
            "search_page",
            "search.list cost=100".to_string(),
            Some(keyword.to_string()),
            None,
        );

        let channel_ids = search_body
            .get("items")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        item.get("snippet")
                            .and_then(|value| value.get("channelId"))
                            .and_then(Value::as_str)
                            .or_else(|| {
                                item.get("id")
                                    .and_then(|value| value.get("channelId"))
                                    .and_then(Value::as_str)
                            })
                            .map(ToOwned::to_owned)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if channel_ids.is_empty() {
            page_token = next_page_token(&search_body);
            if page_token.is_none() {
                break;
            }
            continue;
        }

        sleep_rate(config.rate_limit_ms);
        channel_calls.fetch_add(1, Ordering::SeqCst);
        let channels_body = get_json(
            client,
            &config.api_key,
            "/channels",
            vec![
                ("part", "snippet,statistics,contentDetails".to_string()),
                ("id", channel_ids.join(",")),
            ],
        )?;
        let channel_items = channels_body
            .get("items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let batch_scanned = channel_items.len() as i64;
        keyword_scanned += batch_scanned;
        let global_scanned =
            scanned_total.fetch_add(batch_scanned, Ordering::SeqCst) + batch_scanned;
        handle.push_log(
            &config.platform,
            "channel_batch",
            format!("channels.list size={batch_scanned}"),
            Some(keyword.to_string()),
            None,
        );

        for channel in channel_items {
            if stop_flag.load(Ordering::SeqCst) || handle.cancel_requested.load(Ordering::SeqCst) {
                return Err(CrawlError::Cancelled);
            }
            if reached_max_total(config, accepted_total.load(Ordering::SeqCst)) {
                set_stop_reason(stop_reason, "max_total_reached");
                stop_flag.store(true, Ordering::SeqCst);
                break;
            }

            let country = channel
                .get("snippet")
                .and_then(|value| value.get("country"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_uppercase();
            if !country.is_empty()
                && config
                    .exclude_countries
                    .iter()
                    .any(|value| value.eq_ignore_ascii_case(&country))
            {
                continue;
            }

            let uploads_id = channel
                .get("contentDetails")
                .and_then(|value| value.get("relatedPlaylists"))
                .and_then(|value| value.get("uploads"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let year_videos = if let Some(uploads_id) = uploads_id {
                sleep_rate(config.rate_limit_ms);
                let (count, used_pages) = count_year_videos(client, config, &uploads_id)?;
                playlist_pages.fetch_add(used_pages, Ordering::SeqCst);
                count
            } else {
                0
            };

            if year_videos < config.min_year_video_count {
                continue;
            }

            let record =
                youtube_channel_record(&channel, &config.job_id, keyword, &config.platform);
            if let Err(error) = config.channels.insert_accepted(&record) {
                handle.push_log(
                    &config.platform,
                    "filter",
                    format!("failed to persist channel: {error}"),
                    Some(keyword.to_string()),
                    None,
                );
            } else {
                emit_channel_accepted(handle, &record);
                maybe_enqueue_email_enrich(handle, config, &record);
            }

            keyword_accepted += 1;
            let global_accepted = accepted_total.fetch_add(1, Ordering::SeqCst) + 1;
            let quota_used = calculate_expected_quota(
                search_pages.load(Ordering::SeqCst),
                channel_calls.load(Ordering::SeqCst),
                playlist_pages.load(Ordering::SeqCst),
            );
            handle.set_progress(
                keyword,
                keyword_scanned,
                keyword_accepted,
                global_scanned,
                global_accepted,
                quota_used,
            );
        }

        let quota_used = calculate_expected_quota(
            search_pages.load(Ordering::SeqCst),
            channel_calls.load(Ordering::SeqCst),
            playlist_pages.load(Ordering::SeqCst),
        );
        handle.set_progress(
            keyword,
            keyword_scanned,
            keyword_accepted,
            scanned_total.load(Ordering::SeqCst),
            accepted_total.load(Ordering::SeqCst),
            quota_used,
        );

        page_token = next_page_token(&search_body);
        if page_token.is_none() || stop_flag.load(Ordering::SeqCst) {
            break;
        }
    }

    handle.mark_keyword_done();
    handle.push_log(
        &config.platform,
        "keyword_done",
        format!(
            "keyword done accepted_total={}",
            accepted_total.load(Ordering::SeqCst)
        ),
        Some(keyword.to_string()),
        None,
    );
    Ok(())
}

fn search_channels(
    client: &Client,
    config: &RunConfig,
    keyword: &str,
    page_token: Option<&str>,
) -> Result<Value, CrawlError> {
    let mut query = vec![
        ("part", "snippet".to_string()),
        ("q", keyword.to_string()),
        ("type", "channel".to_string()),
        ("maxResults", "50".to_string()),
    ];
    if let Some(token) = page_token {
        query.push(("pageToken", token.to_string()));
    }
    get_json(client, &config.api_key, "/search", query)
}

fn count_year_videos(
    client: &Client,
    config: &RunConfig,
    uploads_playlist_id: &str,
) -> Result<(i64, i64), CrawlError> {
    let mut count = 0i64;
    let mut pages = 0i64;
    let mut page_token: Option<String> = None;

    loop {
        pages += 1;
        let mut query = vec![
            ("part", "contentDetails".to_string()),
            ("playlistId", uploads_playlist_id.to_string()),
            ("maxResults", "50".to_string()),
        ];
        if let Some(token) = &page_token {
            query.push(("pageToken", token.clone()));
        }
        let body = match get_json(client, &config.api_key, "/playlistItems", query) {
            Ok(value) => value,
            Err(CrawlError::Message(message)) if message.contains("playlistNotFound") => {
                return Ok((0, pages))
            }
            Err(error) => return Err(error),
        };

        let items = body
            .get("items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for item in items {
            let Some(published) = item
                .get("contentDetails")
                .and_then(|value| value.get("videoPublishedAt"))
                .and_then(Value::as_str)
            else {
                continue;
            };
            let year = published
                .get(0..4)
                .and_then(|value| value.parse::<i32>().ok());
            match year {
                Some(value) if value < config.year => return Ok((count, pages)),
                Some(value) if value == config.year => count += 1,
                _ => {}
            }
        }

        page_token = next_page_token(&body);
        if page_token.is_none() {
            break;
        }
    }

    Ok((count, pages))
}

fn get_json(
    client: &Client,
    api_key: &str,
    path: &str,
    params: Vec<(&str, String)>,
) -> Result<Value, CrawlError> {
    let mut query: Vec<(String, String)> = params
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect();
    query.push(("key".to_string(), api_key.to_string()));

    let response = client
        .get(format!("{API_BASE}{path}"))
        .query(&query)
        .send()
        .map_err(|error| CrawlError::Message(format!("YouTube API network error: {error}")))?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| CrawlError::Message(format!("YouTube API read error: {error}")))?;

    if !status.is_success() {
        if is_quota_exceeded(&body) {
            return Err(CrawlError::QuotaExceeded);
        }
        return Err(CrawlError::Message(format!(
            "YouTube API HTTP {}: {}",
            status.as_u16(),
            truncate_body(&body)
        )));
    }

    serde_json::from_str(&body)
        .map_err(|error| CrawlError::Message(format!("invalid YouTube API JSON: {error}")))
}

fn split_csv(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn calculate_expected_quota(
    search_pages: i64,
    channel_calls: i64,
    playlist_item_pages: i64,
) -> i64 {
    (search_pages * 100) + channel_calls + playlist_item_pages
}

fn reached_max_total(config: &RunConfig, accepted: i64) -> bool {
    config.max_total > 0 && accepted >= config.max_total
}

fn next_page_token(body: &Value) -> Option<String> {
    body.get("nextPageToken")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn is_quota_exceeded(body: &str) -> bool {
    let lowered = body.to_ascii_lowercase();
    lowered.contains("quotaexceeded") || lowered.contains("quota exceeded")
}

fn truncate_body(body: &str) -> String {
    body.chars().take(500).collect()
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

fn set_stop_reason(target: &Arc<Mutex<Option<String>>>, reason: &str) {
    if let Ok(mut guard) = target.lock() {
        if guard.is_none() {
            *guard = Some(reason.to_string());
        }
    }
}

fn sleep_rate(rate_limit_ms: u64) {
    if rate_limit_ms > 0 {
        thread::sleep(Duration::from_millis(rate_limit_ms));
    }
}

/// Map a YouTube `channels.list` item into a persistable `ChannelRecord`.
fn youtube_channel_record(
    channel: &Value,
    job_id: &str,
    keyword: &str,
    platform: &str,
) -> ChannelRecord {
    let snippet = channel.get("snippet").and_then(Value::as_object);
    let statistics = channel.get("statistics").and_then(Value::as_object);
    let description = snippet
        .and_then(|value| value.get("description"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let subscriber_count = statistics
        .and_then(|value| value.get("subscriberCount"))
        .and_then(Value::as_str)
        .and_then(|value| value.parse::<i64>().ok());
    let email = extract_email(&description);
    let email_status = ChannelRecord::initial_email_status(&email).to_string();
    ChannelRecord {
        job_id: job_id.to_string(),
        keyword: keyword.to_string(),
        platform: platform.to_string(),
        channel_id: channel
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        title: snippet
            .and_then(|value| value.get("title"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        country: snippet
            .and_then(|value| value.get("country"))
            .and_then(Value::as_str)
            .map(str::to_string),
        subscriber_count,
        email,
        verified_email: None,
        description: Some(description),
        custom_url: snippet
            .and_then(|value| value.get("customUrl"))
            .and_then(Value::as_str)
            .map(str::to_string),
        email_status,
        enrich_attempts: 0,
        enrich_error: None,
        enriched_at: None,
    }
}

/// Best-effort email extraction from channel description (no regex dependency).
fn extract_email(description: &str) -> Option<String> {
    let normalized = description
        .replace("[at]", "@")
        .replace("(at)", "@")
        .replace("[dot]", ".")
        .replace("(dot)", ".");
    let mut token = String::new();
    for ch in normalized.chars() {
        if ch.is_whitespace() {
            if token.contains('@') && token.contains('.') && token.len() >= 5 {
                return Some(token);
            }
            token.clear();
        } else {
            token.push(ch);
        }
    }
    if token.contains('@') && token.contains('.') && token.len() >= 5 {
        Some(token)
    } else {
        None
    }
}

/// Push [`crate::CrawlerUiEvent::ChannelAccepted`] for UI list updates without polling.
///
/// 作者：coisini
/// 创建时间：2026-07-21
fn emit_channel_accepted(handle: &JobHandle, record: &ChannelRecord) {
    handle
        .emitter
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

/// Enqueue email enrich when description had no email — never fails the crawl.
///
/// 作者：coisini
/// 创建时间：2026-07-21
fn maybe_enqueue_email_enrich(handle: &JobHandle, config: &RunConfig, record: &ChannelRecord) {
    if record.email_status != EMAIL_STATUS_PENDING_ENRICH {
        return;
    }
    let Some(jobs_queue) = config.jobs_queue.as_ref() else {
        handle.push_log(
            &config.platform,
            "enrich_skip",
            format!(
                "email enrich queue unavailable; channel={} kept",
                record.channel_id
            ),
            Some(record.keyword.clone()),
            None,
        );
        return;
    };

    let payload = CrawlerEmailEnrichPayload {
        crawler_job_id: record.job_id.clone(),
        channel_id: record.channel_id.clone(),
        platform: record.platform.clone(),
        custom_url: record.custom_url.clone(),
        title: record.title.clone(),
        attempt: 1,
    };
    let payload_json = match serde_json::to_string(&payload) {
        Ok(value) => value,
        Err(error) => {
            handle.push_log(
                &config.platform,
                "enrich_skip",
                format!("enrich payload serialize failed: {error}"),
                Some(record.keyword.clone()),
                None,
            );
            return;
        }
    };

    match jobs_queue.enqueue(JOB_TYPE_CRAWLER_EMAIL_ENRICH, &payload_json) {
        Ok(job_id) => {
            handle.push_log(
                &config.platform,
                "enrich_enqueued",
                format!(
                    "email enrich queued background_job={job_id} channel={}",
                    record.channel_id
                ),
                Some(record.keyword.clone()),
                None,
            );
        }
        Err(error) => {
            // Side path only — main crawl must continue.
            handle.push_log(
                &config.platform,
                "enrich_skip",
                format!(
                    "email enrich enqueue failed (non-blocking): {error}; channel={} kept",
                    record.channel_id
                ),
                Some(record.keyword.clone()),
                None,
            );
        }
    }
}
