//! 发布网关适配器 — 内存实现 [`crate::publish::PublishGateway`]。
//!
//! 组合已有存储：账号校验读 `InMemoryAccountStore`、发布日志写 `InMemoryPublishLogStore`。
//! 实际平台发布（闲鱼签名 API）属 sidecar 职责，此处以模拟成功返回 item_url/item_id，
//! 保证 `PublishService` 编排链路真实可走通。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use std::sync::{Arc, Mutex};

use crate::account::XianyuAccount;
use crate::publish::gateway::PublishLogEntry;
use crate::publish::{
    AccountCapability, PublishGateway, PublishLog, PublishLogStatus, PublishResult, SyncInfo,
};
use crate::AccountStore;
use async_trait::async_trait;
use common::DingDaError;

use super::stores::{append_log, update_log, InMemoryAccountStore, InMemoryPublishLogStore};

/// 内存发布网关（单实例共享，供 `PublishService` 使用）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
pub struct InMemoryPublishGateway {
    accounts: Arc<InMemoryAccountStore>,
    logs: Arc<InMemoryPublishLogStore>,
    next_log_id: Arc<Mutex<i64>>,
}

impl InMemoryPublishGateway {
    /// 构建网关。
    pub fn new(accounts: Arc<InMemoryAccountStore>, logs: Arc<InMemoryPublishLogStore>) -> Self {
        Self {
            accounts,
            logs,
            next_log_id: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl PublishGateway for InMemoryPublishGateway {
    fn account_cookie(
        &self,
        user_id: i64,
        account_id: &str,
    ) -> crate::DingDaResult<Option<String>> {
        let account = self.accounts.get_account(user_id, account_id)?;
        Ok(account
            .filter(|account: &XianyuAccount| !account.cookie.is_empty())
            .map(|account| account.cookie.clone()))
    }

    fn resolve_address(
        &self,
        _account_id: &str,
        item: &serde_json::Value,
    ) -> crate::DingDaResult<serde_json::Value> {
        Ok(item.clone())
    }

    async fn detect_capability(
        &self,
        account_id: &str,
        _cookie: &str,
        user_id: i64,
    ) -> crate::DingDaResult<AccountCapability> {
        let exists = self.accounts.get_account(user_id, account_id)?.is_some();
        if !exists {
            return Ok(AccountCapability {
                success: false,
                is_fish_shop: false,
                cookies_str: None,
                message: "账号不存在或无权限".to_string(),
            });
        }
        Ok(AccountCapability {
            success: true,
            is_fish_shop: true,
            cookies_str: None,
            message: "能力检测通过（内存网关：默认鱼小铺）".to_string(),
        })
    }

    async fn publish_fish_shop(
        &self,
        item: &serde_json::Value,
        _cookie: &str,
        _account_id: &str,
        _user_id: i64,
    ) -> crate::DingDaResult<PublishResult> {
        let title = item
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("商品");
        let fake_id = format!("mnt{}", uuid_fragment());
        Ok(PublishResult {
            success: true,
            item_url: Some(format!(
                "{}{fake_id}",
                common::constants::xianyu::ITEM_URL_PREFIX
            )),
            item_id: Some(fake_id),
            message: format!("内存网关：商品「{title}」发布成功（模拟）"),
            cookies_str: None,
            account_invalid: false,
        })
    }

    async fn publish_personal(
        &self,
        item: &serde_json::Value,
        _cookie: &str,
        _account_id: &str,
        _user_id: i64,
    ) -> crate::DingDaResult<PublishResult> {
        self.publish_fish_shop(item, "", "", 0).await
    }

    fn create_log(&self, entry: &PublishLogEntry, status: &str) -> crate::DingDaResult<i64> {
        let mut next = self
            .next_log_id
            .lock()
            .map_err(|error| DingDaError::internal(error.to_string()))?;
        *next += 1;
        let log_id = *next;
        let log = PublishLog {
            id: log_id,
            owner_id: entry.user_id,
            account_id: entry.account_id.clone(),
            title: entry.title.clone(),
            price: Some(entry.price.clone()),
            status: status_from_str(status),
            item_url: None,
            item_id: None,
            error_message: None,
            resolved_address_text: None,
            address_source: None,
            created_at: Some(now_string()),
        };
        append_log(&self.logs, log)?;
        Ok(log_id)
    }

    fn update_log(
        &self,
        log_id: i64,
        status: &str,
        item_url: Option<&str>,
        item_id: Option<&str>,
        error_message: Option<&str>,
    ) -> crate::DingDaResult<()> {
        update_log(
            &self.logs,
            log_id,
            status_from_str(status),
            item_url,
            item_id,
            error_message,
        )
    }

    async fn sync_account_items(
        &self,
        _account_id: &str,
        _cookie: &str,
    ) -> crate::DingDaResult<SyncInfo> {
        Ok(SyncInfo {
            sync_status: "skipped".to_string(),
            sync_message: "内存网关：商品同步由 sidecar 执行".to_string(),
            sync_total_count: 0,
            sync_saved_count: 0,
        })
    }
}

fn status_from_str(status: &str) -> PublishLogStatus {
    match status {
        "success" => PublishLogStatus::Success,
        "failed" => PublishLogStatus::Failed,
        _ => PublishLogStatus::Pending,
    }
}

fn uuid_fragment() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
        .to_string()
}

fn now_string() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
