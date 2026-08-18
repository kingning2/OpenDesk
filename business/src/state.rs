//! 全局应用状态 — 数据库连接池、账号存储、任务注册表。
//!
//! 供桌面壳层 setup 组装并注入 Tauri State；不依赖 Tauri 类型。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use crate::AccountStore;
use std::sync::Arc;

use crate::xianyu::SqliteBusinessDb;

/// 业务 SQLite 连接池（当前为单库 `Arc` 包装，后续可换连接池实现）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
pub type DbPool = Arc<SqliteBusinessDb>;

/// 后台任务注册表（占位 — 后续接入 `kernel::task` 调度器）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Debug, Default)]
pub struct TaskRegistry {
    // 预留：task_id → JoinHandle / 元数据
}

impl TaskRegistry {
    /// 创建空注册表。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-18
    pub fn new() -> Self {
        Self::default()
    }
}

/// 全局应用状态。
///
/// 功能：
/// - 持有业务库 [`DbPool`]
/// - 持有账号存储（[`AccountStore`] trait object）
/// - 持有任务注册表 [`TaskRegistry`]（当前为空实现）
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
pub struct AppState {
    /// 业务 SQLite 库。
    pub db: DbPool,
    /// 账号存储（闲鱼等领域共用 Port）。
    pub accounts: Arc<dyn AccountStore + Send + Sync>,
    /// 后台任务注册表。
    pub tasks: TaskRegistry,
}

impl AppState {
    /// 组装应用状态。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-18
    ///
    /// # 参数
    /// - `db` — 已打开的业务库
    /// - `accounts` — 账号存储实现
    pub fn new(db: DbPool, accounts: Arc<dyn AccountStore + Send + Sync>) -> Self {
        Self {
            db,
            accounts,
            tasks: TaskRegistry::new(),
        }
    }

    /// 只读访问数据库池。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-18
    pub fn db(&self) -> &SqliteBusinessDb {
        &self.db
    }

    /// 只读访问账号存储。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-18
    pub fn account_store(&self) -> &dyn AccountStore {
        self.accounts.as_ref()
    }
}
