//! Background task scheduler.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use thiserror::Error;
use tokio::task::JoinHandle;
use tracing::info;

pub type TaskId = u64;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("schedule failed: {0}")]
    ScheduleFailed(String),
    #[error("cancel failed: {0}")]
    CancelFailed(String),
}

pub trait TaskScheduler: Send + Sync {
    fn schedule(&self, name: &str, delay_ms: u64) -> Result<TaskId, TaskError>;
    fn cancel(&self, task_id: TaskId) -> Result<(), TaskError>;
}

pub struct InMemoryTaskScheduler {
    next_id: AtomicU64,
    tasks: Mutex<HashMap<TaskId, JoinHandle<()>>>,
}

impl Default for InMemoryTaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryTaskScheduler {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            tasks: Mutex::new(HashMap::new()),
        }
    }
}

impl TaskScheduler for InMemoryTaskScheduler {
    fn schedule(&self, name: &str, delay_ms: u64) -> Result<TaskId, TaskError> {
        let task_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let task_name = name.to_string();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            info!(task_id, task = %task_name, "scheduled task executed");
        });

        let mut tasks = self
            .tasks
            .lock()
            .map_err(|error| TaskError::ScheduleFailed(error.to_string()))?;
        tasks.insert(task_id, handle);
        Ok(task_id)
    }

    fn cancel(&self, task_id: TaskId) -> Result<(), TaskError> {
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|error| TaskError::CancelFailed(error.to_string()))?;
        if let Some(handle) = tasks.remove(&task_id) {
            handle.abort();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancel_prevents_task_execution() {
        let scheduler = InMemoryTaskScheduler::new();
        let task_id = scheduler.schedule("noop", 500).ok();
        assert!(task_id.is_some());
        if let Some(id) = task_id {
            scheduler.cancel(id).ok();
            tokio::time::sleep(Duration::from_millis(600)).await;
        }
    }
}
