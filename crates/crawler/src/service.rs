//! 采集服务入口与任务线程编排。

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

use common::contracts::{
    CrawlerIpcJobCancelRequest, CrawlerIpcJobCancelResponse, CrawlerIpcJobLogsRequest,
    CrawlerIpcJobLogsResponse, CrawlerIpcJobStartRequest, CrawlerIpcJobStartResponse,
    CrawlerIpcJobStatusRequest, CrawlerIpcJobStatusResponse,
};
use common::i18n::Locale;
use ports::background_job::BackgroundJobStore;
use ports::crawler_channels::CrawlerChannelStore;
use reqwest::blocking::Client;
use uuid::Uuid;

use crate::job::JobHandle;
use crate::youtube::{
    calculate_expected_quota, crawl_keyword, reached_max_total, set_stop_reason, CrawlError,
};
use crate::{CrawlerUiEmitter, NoopCrawlerUiEmitter};

const USER_AGENT: &str = "OpenDeskCrawler/0.1";

/// 按任务 ID 管理进程内 YouTube 采集任务。
#[derive(Clone)]
pub struct CrawlerService {
    jobs: Arc<Mutex<HashMap<String, Arc<JobHandle>>>>,
    channels: Arc<dyn CrawlerChannelStore>,
    emitter: Arc<Mutex<Arc<dyn CrawlerUiEmitter>>>,
    jobs_queue: Arc<Mutex<Option<Arc<dyn BackgroundJobStore>>>>,
}

#[derive(Clone)]
pub(super) struct RunConfig {
    pub(super) job_id: String,
    pub(super) platform: String,
    pub(super) api_key: String,
    pub(super) keywords: Vec<String>,
    pub(super) rate_limit_ms: u64,
    pub(super) max_total: i64,
    pub(super) year: i32,
    pub(super) min_year_video_count: i64,
    pub(super) exclude_countries: Vec<String>,
    pub(super) channels: Arc<dyn CrawlerChannelStore>,
    pub(super) jobs_queue: Option<Arc<dyn BackgroundJobStore>>,
}

impl CrawlerService {
    /// 创建使用空 UI 事件接收器的采集服务。
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

    /// 设置后续采集任务使用的 UI 事件接收器。
    pub fn attach_emitter(&self, emitter: Arc<dyn CrawlerUiEmitter>) {
        if let Ok(mut slot) = self.emitter.lock() {
            *slot = emitter;
        }
    }

    /// 设置邮箱补全任务使用的后台任务存储。
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

    /// 启动进程内 YouTube 采集任务。
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
            emitter,
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

    /// 请求取消指定采集任务。
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

    /// 查询指定采集任务的最新状态。
    pub fn status(
        &self,
        request: CrawlerIpcJobStatusRequest,
    ) -> Result<CrawlerIpcJobStatusResponse, String> {
        let handle = self.job(&request.job_id)?;
        let mut response = handle.status_response();
        response.trace_id = request.trace_id;
        Ok(response)
    }

    /// 查询指定采集任务的进程内日志。
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

    fn run_job(&self, handle: Arc<JobHandle>, config: RunConfig) {
        // 1. 初始化任务状态和共享计数器。
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

        // 2. 最多启动四个 worker，通过原子索引领取关键词。
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

        // 3. 汇总 worker 结果，保留首个业务失败或 panic。
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

        // 4. 将汇总结果映射为唯一终态并发出事件。
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

fn split_csv(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}
