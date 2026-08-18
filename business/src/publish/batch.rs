//! 批量发布 — 任务模型与进度追踪。
//!
//! 对齐 Python 版 `publish_batch` 语义：
//! - 任务 = 多账号 × 多素材 的笛卡尔积发布；
//! - 提交创建任务（pending = 账号数 × 素材数），后台逐条执行；
//! - 按账号统计进度（成功/失败/进行中/待处理）+ 发布后商品同步状态。

use common::OpenDeskResult;
use serde::{Deserialize, Serialize};

/// 单账号发布统计 + 商品同步状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAccountStatus {
    pub account_id: String,
    pub total: u32,
    pub success: u32,
    pub failed: u32,
    pub publishing: u32,
    pub pending: u32,
    #[serde(default)]
    pub sync_status: String,
    #[serde(default)]
    pub sync_message: String,
    #[serde(default)]
    pub sync_total_count: u32,
    #[serde(default)]
    pub sync_saved_count: u32,
}

impl BatchAccountStatus {
    pub fn new(account_id: &str, total: u32) -> Self {
        Self {
            account_id: account_id.to_string(),
            total,
            success: 0,
            failed: 0,
            publishing: 0,
            pending: total,
            sync_status: "pending".to_string(),
            sync_message: "等待该账号发布完成后自动获取商品".to_string(),
            sync_total_count: 0,
            sync_saved_count: 0,
        }
    }
}

/// 批量发布任务。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTask {
    pub batch_id: String,
    pub owner_id: i64,
    pub account_ids: Vec<String>,
    pub material_ids: Vec<i64>,
    pub total: u32,
    pub success: u32,
    pub failed: u32,
    pub publishing: u32,
    pub pending: u32,
    pub finished: bool,
    pub account_statuses: Vec<BatchAccountStatus>,
}

impl BatchTask {
    /// 由账号 × 素材笛卡尔积构造初始任务。
    pub fn new(
        batch_id: String,
        owner_id: i64,
        account_ids: &[String],
        material_ids: &[i64],
    ) -> Self {
        let total = (account_ids.len() * material_ids.len()) as u32;
        let account_statuses = account_ids
            .iter()
            .map(|account_id| BatchAccountStatus::new(account_id, material_ids.len() as u32))
            .collect();
        Self {
            batch_id,
            owner_id,
            account_ids: account_ids.to_vec(),
            material_ids: material_ids.to_vec(),
            total,
            success: 0,
            failed: 0,
            publishing: 0,
            pending: total,
            finished: false,
            account_statuses,
        }
    }

    /// 记录一条发布结果（更新总计数 + 账号计数）。
    pub fn record(&mut self, account_id: &str, ok: bool) {
        if ok {
            self.success += 1;
        } else {
            self.failed += 1;
        }
        self.publishing = self.publishing.saturating_sub(1);
        self.pending = self.pending.saturating_sub(1);
        if let Some(status) = self
            .account_statuses
            .iter_mut()
            .find(|s| s.account_id == account_id)
        {
            if ok {
                status.success += 1;
            } else {
                status.failed += 1;
            }
            status.publishing = status.publishing.saturating_sub(1);
            status.pending = status.pending.saturating_sub(1);
        }
        self.finished = self.success + self.failed == self.total;
    }

    /// 标记账号同步结果。
    pub fn mark_sync(
        &mut self,
        account_id: &str,
        status: &str,
        message: &str,
        total: u32,
        saved: u32,
    ) {
        if let Some(account) = self
            .account_statuses
            .iter_mut()
            .find(|s| s.account_id == account_id)
        {
            account.sync_status = status.to_string();
            account.sync_message = message.to_string();
            account.sync_total_count = total;
            account.sync_saved_count = saved;
        }
    }
}

/// 批量任务存储 Port。
pub trait BatchStore: Send + Sync {
    /// 创建任务（同 batch_id 覆盖）。
    fn create_task(&self, task: &BatchTask) -> OpenDeskResult<()>;

    /// 按归属查询任务。
    fn get_task(&self, owner_id: i64, batch_id: &str) -> OpenDeskResult<Option<BatchTask>>;

    /// 更新任务进度。
    fn update_task(&self, task: &BatchTask) -> OpenDeskResult<()>;
}

/// 批量发布服务。
pub struct BatchService<'a> {
    store: &'a dyn BatchStore,
}

impl<'a> BatchService<'a> {
    pub fn new(store: &'a dyn BatchStore) -> Self {
        Self { store }
    }

    /// 提交任务（账号与素材均非空；batch_id 由调用方生成）。
    pub fn submit(
        &self,
        batch_id: &str,
        owner_id: i64,
        account_ids: &[String],
        material_ids: &[i64],
    ) -> OpenDeskResult<BatchTask> {
        if account_ids.is_empty() {
            return Err("请至少选择一个账号".to_string().into());
        }
        if material_ids.is_empty() {
            return Err("请至少选择一条素材".to_string().into());
        }
        let task = BatchTask::new(batch_id.to_string(), owner_id, account_ids, material_ids);
        self.store.create_task(&task)?;
        Ok(task)
    }

    /// 查询任务进度。
    pub fn status(&self, owner_id: i64, batch_id: &str) -> OpenDeskResult<Option<BatchTask>> {
        self.store.get_task(owner_id, batch_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockStore {
        tasks: Mutex<HashMap<String, BatchTask>>,
    }

    impl BatchStore for MockStore {
        fn create_task(&self, task: &BatchTask) -> OpenDeskResult<()> {
            self.tasks
                .lock()
                .expect("lock")
                .insert(task.batch_id.clone(), task.clone());
            Ok(())
        }
        fn get_task(&self, owner_id: i64, batch_id: &str) -> OpenDeskResult<Option<BatchTask>> {
            Ok(self
                .tasks
                .lock()
                .expect("lock")
                .get(batch_id)
                .filter(|task| task.owner_id == owner_id)
                .cloned())
        }
        fn update_task(&self, task: &BatchTask) -> OpenDeskResult<()> {
            self.tasks
                .lock()
                .expect("lock")
                .insert(task.batch_id.clone(), task.clone());
            Ok(())
        }
    }

    #[test]
    fn submit_requires_accounts_and_materials() {
        let store = MockStore {
            tasks: Mutex::new(HashMap::new()),
        };
        let service = BatchService::new(&store);
        assert!(service.submit("b1", 1, &[], &[1]).is_err());
        assert!(service
            .submit("b1", 1, &["acc-1".to_string()], &[])
            .is_err());
        assert!(service
            .submit("b1", 1, &["acc-1".to_string()], &[1, 2])
            .is_ok());
    }

    #[test]
    fn task_progress_records_results() {
        let mut task = BatchTask::new(
            "b1".to_string(),
            1,
            &["acc-1".to_string(), "acc-2".to_string()],
            &[1, 2],
        );
        assert_eq!(task.total, 4);
        assert_eq!(task.pending, 4);
        task.record("acc-1", true);
        task.record("acc-1", false);
        assert_eq!(task.success, 1);
        assert_eq!(task.failed, 1);
        assert_eq!(task.pending, 2);
        assert!(!task.finished);
        task.record("acc-2", true);
        task.record("acc-2", true);
        assert!(task.finished);
        assert_eq!(task.success, 3);
        let acc1 = task
            .account_statuses
            .iter()
            .find(|s| s.account_id == "acc-1")
            .expect("acc1");
        assert_eq!(acc1.success, 1);
        assert_eq!(acc1.failed, 1);
    }

    #[test]
    fn status_respects_ownership() {
        let store = MockStore {
            tasks: Mutex::new(HashMap::new()),
        };
        let service = BatchService::new(&store);
        service
            .submit("b1", 1, &["acc-1".to_string()], &[1])
            .expect("submit");
        assert!(service.status(2, "b1").expect("status").is_none());
        assert!(service.status(1, "b1").expect("status").is_some());
    }
}
