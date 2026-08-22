//! 闲鱼监控定时调度 — 多任务并发（Semaphore 限流）。

use chrono::{DateTime, Utc};
use platform::domain::monitor::{MonitorTask, MonitorTaskStore};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{sleep, Duration};

use super::engine::MonitorEngine;

const TICK_SECONDS: u64 = 30;
const MAX_CONCURRENT: usize = 2;

pub struct MonitorScheduler {
    engine: Arc<MonitorEngine>,
    owner_id: i64,
    running: Arc<Mutex<HashSet<String>>>,
    semaphore: Arc<Semaphore>,
}

impl MonitorScheduler {
    pub fn new(engine: Arc<MonitorEngine>, owner_id: i64) -> Self {
        Self {
            engine,
            owner_id,
            running: Arc::new(Mutex::new(HashSet::new())),
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT)),
        }
    }

    pub fn start(self: Arc<Self>) {
        tauri::async_runtime::spawn(async move {
            loop {
                if let Err(error) = self.tick().await {
                    warn!(%error, "监控调度 tick 失败");
                }
                sleep(Duration::from_secs(TICK_SECONDS)).await;
            }
        });
    }

    async fn tick(&self) -> common::DingDaResult<()> {
        let tasks = self.engine.tasks.list_tasks(self.owner_id)?;
        for task in tasks
            .into_iter()
            .filter(|task| task.enabled && !task.is_running)
        {
            if !self.is_due(&task) {
                continue;
            }
            if self.running.lock().await.contains(&task.id) {
                continue;
            }
            self.spawn_run(task.id);
        }
        Ok(())
    }

    fn is_due(&self, task: &MonitorTask) -> bool {
        let interval = task.interval_minutes.max(1) as i64;
        let Some(last_run) = task.last_run_at.as_deref() else {
            return true;
        };
        let Ok(parsed) = DateTime::parse_from_rfc3339(last_run) else {
            return true;
        };
        let elapsed = Utc::now().signed_duration_since(parsed.with_timezone(&Utc));
        elapsed.num_minutes() >= interval
    }

    fn spawn_run(&self, task_id: String) {
        let engine = self.engine.clone();
        let running = self.running.clone();
        let semaphore = self.semaphore.clone();
        let owner_id = self.owner_id;
        tauri::async_runtime::spawn(async move {
            running.lock().await.insert(task_id.clone());
            let _permit = semaphore.acquire().await.ok();
            let result = engine.run_task(owner_id, &task_id).await;
            if let Err(error) = result {
                warn!(task_id = %task_id, %error, "监控任务执行失败");
            }
            running.lock().await.remove(&task_id);
        });
    }
}
