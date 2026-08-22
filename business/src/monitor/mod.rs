//! 闲鱼商品监控 — 任务与结果模型、SQLite Port、业务服务。
//!
//! 定时调度 / Sidecar 搜索 / AI 决策由 Tauri 壳层编排；本模块仅持久化与查询。

use common::DingDaResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 监控任务。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorTask {
    pub id: String,
    pub owner_id: i64,
    pub name: String,
    /// 用户意图描述，供 AI 生成搜索关键词。
    pub intent: String,
    /// AI 生成或人工校正后的关键词列表。
    pub keywords: Vec<String>,
    pub account_id: String,
    /// 全局 AI 配置中的账号 id；无账号平台用 `provider:{id}`（如 `provider:ollama`）。
    #[serde(default)]
    pub ai_account_id: String,
    /// 首选账号失败后是否自动切换备用 AI 账号。
    #[serde(default = "default_ai_failover_enabled")]
    pub ai_failover_enabled: bool,
    /// 备用 AI 账号顺序（不含首选）；为空则按 AI 配置自动推断。
    #[serde(default)]
    pub ai_account_order: Vec<String>,
    /// 定时间隔（分钟）。
    pub interval_minutes: u32,
    pub enabled: bool,
    /// AI 决策标准（自然语言）。
    pub ai_criteria: String,
    pub max_results: u32,
    /// 有头浏览器搜索（默认 true）。
    #[serde(default = "default_headed")]
    pub headed: bool,
    pub is_running: bool,
    pub last_run_at: Option<String>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn default_headed() -> bool {
    true
}

fn default_ai_failover_enabled() -> bool {
    true
}

/// 监控命中结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorResult {
    pub id: String,
    pub task_id: String,
    pub owner_id: i64,
    pub item_id: String,
    pub title: String,
    pub url: String,
    pub price_text: String,
    pub location: String,
    pub seller_name: String,
    pub ai_recommended: bool,
    pub ai_reason: String,
    pub notified: bool,
    #[serde(default)]
    pub raw: Value,
    pub crawled_at: String,
}

/// 监控任务存储 Port。
pub trait MonitorTaskStore: Send + Sync {
    fn list_tasks(&self, owner_id: i64) -> DingDaResult<Vec<MonitorTask>>;
    fn get_task(&self, owner_id: i64, task_id: &str) -> DingDaResult<Option<MonitorTask>>;
    fn put_task(&self, task: &MonitorTask) -> DingDaResult<()>;
    fn delete_task(&self, owner_id: i64, task_id: &str) -> DingDaResult<()>;
}

/// 监控结果存储 Port。
pub trait MonitorResultStore: Send + Sync {
    fn list_results(&self, owner_id: i64, task_id: &str) -> DingDaResult<Vec<MonitorResult>>;
    fn has_result(&self, owner_id: i64, task_id: &str, item_id: &str) -> DingDaResult<bool>;
    fn put_result(&self, result: &MonitorResult) -> DingDaResult<()>;
}

/// 监控业务服务。
pub struct MonitorService<'a> {
    tasks: &'a dyn MonitorTaskStore,
    results: &'a dyn MonitorResultStore,
}

impl<'a> MonitorService<'a> {
    pub fn new(tasks: &'a dyn MonitorTaskStore, results: &'a dyn MonitorResultStore) -> Self {
        Self { tasks, results }
    }

    pub fn list_tasks(&self, owner_id: i64) -> DingDaResult<Vec<MonitorTask>> {
        self.tasks.list_tasks(owner_id)
    }

    pub fn get_task(&self, owner_id: i64, task_id: &str) -> DingDaResult<Option<MonitorTask>> {
        self.tasks.get_task(owner_id, task_id)
    }

    pub fn save_task(&self, task: &MonitorTask) -> DingDaResult<()> {
        self.tasks.put_task(task)
    }

    pub fn delete_task(&self, owner_id: i64, task_id: &str) -> DingDaResult<()> {
        self.tasks.delete_task(owner_id, task_id)
    }

    pub fn list_results(&self, owner_id: i64, task_id: &str) -> DingDaResult<Vec<MonitorResult>> {
        let mut items = self.results.list_results(owner_id, task_id)?;
        items.sort_by(|a, b| b.crawled_at.cmp(&a.crawled_at));
        Ok(items)
    }

    pub fn has_seen(&self, owner_id: i64, task_id: &str, item_id: &str) -> DingDaResult<bool> {
        self.results.has_result(owner_id, task_id, item_id)
    }

    pub fn save_result(&self, result: &MonitorResult) -> DingDaResult<()> {
        self.results.put_result(result)
    }
}
