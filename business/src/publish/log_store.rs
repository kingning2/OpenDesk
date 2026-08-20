//! 发布日志 — 商品发布记录查询与清空。
//!
//! 对齐 Python 版 `/api/v1/product-publish/logs`：
//! - 分页查询（账号 / 状态筛选）；
//! - 清空 N 天前的日志（默认保留最近 10 天）。

use common::DingDaResult;
use serde::{Deserialize, Serialize};

/// 发布状态。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublishLogStatus {
    #[default]
    Pending,
    Publishing,
    Success,
    Failed,
}

impl PublishLogStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PublishLogStatus::Pending => "pending",
            PublishLogStatus::Publishing => "publishing",
            PublishLogStatus::Success => "success",
            PublishLogStatus::Failed => "failed",
        }
    }
}

/// 发布日志条目（对齐 Python `PublishLog` 核心字段）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishLog {
    pub id: i64,
    pub owner_id: i64,
    pub account_id: String,
    pub title: String,
    #[serde(default)]
    pub price: Option<String>,
    #[serde(default)]
    pub status: PublishLogStatus,
    #[serde(default)]
    pub item_url: Option<String>,
    #[serde(default)]
    pub item_id: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub resolved_address_text: Option<String>,
    /// material / account_pool / global_pool / personal_pool。
    #[serde(default)]
    pub address_source: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// 日志查询条件。
#[derive(Debug, Clone, Default)]
pub struct PublishLogQuery {
    pub page: u32,
    pub page_size: u32,
    pub account_id: String,
    pub status: String,
}

/// 日志存储 Port。
pub trait PublishLogStore: Send + Sync {
    /// 分页查询。
    fn list_logs(&self, owner_id: i64, query: &PublishLogQuery) -> DingDaResult<Vec<PublishLog>>;

    /// 清空 N 天前的日志（days=0 清空全部；created_at 为空视为保留）。
    fn clear_older_than(&self, owner_id: i64, days: u32) -> DingDaResult<()>;
}

/// 日志服务。
pub struct PublishLogService<'a> {
    store: &'a dyn PublishLogStore,
}

impl<'a> PublishLogService<'a> {
    pub fn new(store: &'a dyn PublishLogStore) -> Self {
        Self { store }
    }

    /// 分页查询。
    pub fn list(
        &self,
        owner_id: i64,
        query: &PublishLogQuery,
    ) -> DingDaResult<(Vec<PublishLog>, u32)> {
        let all = self.store.list_logs(owner_id, query)?;
        let total = all.len() as u32;
        Ok((all, total))
    }

    /// 清空 N 天前日志。
    pub fn clear_older_than(&self, owner_id: i64, days: u32) -> DingDaResult<()> {
        self.store.clear_older_than(owner_id, days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockStore {
        logs: Mutex<Vec<PublishLog>>,
    }

    impl PublishLogStore for MockStore {
        fn list_logs(
            &self,
            owner_id: i64,
            query: &PublishLogQuery,
        ) -> DingDaResult<Vec<PublishLog>> {
            let logs = self.logs.lock().expect("lock");
            Ok(logs
                .iter()
                .filter(|log| {
                    log.owner_id == owner_id
                        && (query.account_id.is_empty() || log.account_id == query.account_id)
                        && (query.status.is_empty() || log.status.as_str() == query.status)
                })
                .cloned()
                .collect())
        }
        fn clear_older_than(&self, owner_id: i64, days: u32) -> DingDaResult<()> {
            let mut logs = self.logs.lock().expect("lock");
            let cutoff = if days == 0 {
                None
            } else {
                let now = chrono::Utc::now().date_naive();
                Some((now - chrono::Duration::days(days as i64)).to_string())
            };
            logs.retain(|log| {
                log.owner_id != owner_id
                    || cutoff
                        .as_deref()
                        .is_none_or(|c| log.created_at.as_deref().is_none_or(|t| t >= c))
            });
            Ok(())
        }
    }

    fn log(id: i64, account_id: &str, status: PublishLogStatus, created_at: &str) -> PublishLog {
        PublishLog {
            id,
            owner_id: 1,
            account_id: account_id.to_string(),
            title: format!("商品 {id}"),
            price: Some("99.00".to_string()),
            status,
            item_url: None,
            item_id: None,
            error_message: None,
            resolved_address_text: None,
            address_source: None,
            created_at: Some(created_at.to_string()),
        }
    }

    #[test]
    fn list_filters_by_account_and_status() {
        let store = MockStore {
            logs: Mutex::new(vec![
                log(1, "acc-1", PublishLogStatus::Success, "2026-08-01 10:00:00"),
                log(2, "acc-1", PublishLogStatus::Failed, "2026-08-02 10:00:00"),
                log(3, "acc-2", PublishLogStatus::Success, "2026-08-03 10:00:00"),
            ]),
        };
        let service = PublishLogService::new(&store);
        let query = PublishLogQuery {
            page: 1,
            page_size: 20,
            account_id: "acc-1".to_string(),
            status: String::new(),
        };
        let (list, total) = service.list(1, &query).expect("list");
        assert_eq!(total, 2);
        assert_eq!(list.len(), 2);
        let failed = PublishLogQuery {
            status: "failed".to_string(),
            ..query
        };
        assert_eq!(service.list(1, &failed).expect("list").1, 1);
    }

    #[test]
    fn clear_older_than_keeps_recent() {
        // 用相对日期避免固定时间戳随当前日期推进而过期（8/10 距今 >7 天时用例会挂）。
        let now = chrono::Utc::now().date_naive();
        let old = (now - chrono::Duration::days(10))
            .format("%Y-%m-%d 10:00:00")
            .to_string();
        let recent = (now - chrono::Duration::days(1))
            .format("%Y-%m-%d 10:00:00")
            .to_string();
        let store = MockStore {
            logs: Mutex::new(vec![
                log(1, "acc-1", PublishLogStatus::Success, &old),
                log(2, "acc-1", PublishLogStatus::Success, &recent),
            ]),
        };
        let service = PublishLogService::new(&store);
        // days=7：仅清掉 7 天前的（old 距今 10 天会删，recent 距今 1 天保留）。
        service.clear_older_than(1, 7).expect("clear");
        let (list, total) = service.list(1, &PublishLogQuery::default()).expect("list");
        assert_eq!(total, 1);
        assert_eq!(list[0].id, 2);
    }
}
