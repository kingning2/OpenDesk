//! Web 服务状态 — 内存账号存储 + 应用状态。
//!
//! 与壳层（Tauri）同构：业务校验走 `crates/app` 的 `AccountService`，
//! 存储先用内存实现（进程内），后续换 SQLite。

use app::account::{AccountStore, XianyuAccount};
use common::OpenDeskResult;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 内存账号存储（Web 服务进程内共享）。
#[derive(Clone, Default)]
pub struct InMemoryAccountStore {
    accounts: Arc<Mutex<HashMap<String, XianyuAccount>>>,
    next_id: Arc<Mutex<i64>>,
}

impl InMemoryAccountStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AccountStore for InMemoryAccountStore {
    fn get_account(
        &self,
        owner_id: i64,
        account_id: &str,
    ) -> OpenDeskResult<Option<XianyuAccount>> {
        let accounts = self.accounts.lock().map_err(|e| e.to_string())?;
        Ok(accounts
            .get(account_id)
            .filter(|account| account.owner_id == owner_id)
            .cloned())
    }

    fn list_accounts(&self, owner_id: i64) -> OpenDeskResult<Vec<XianyuAccount>> {
        let accounts = self.accounts.lock().map_err(|e| e.to_string())?;
        Ok(accounts
            .values()
            .filter(|account| account.owner_id == owner_id)
            .cloned()
            .collect())
    }

    fn create_account(&self, account: &XianyuAccount) -> OpenDeskResult<XianyuAccount> {
        let mut accounts = self.accounts.lock().map_err(|e| e.to_string())?;
        if accounts.contains_key(&account.account_id) {
            return Err(format!("账号 {} 已存在", account.account_id).into());
        }
        let mut account = account.clone();
        let mut next_id = self.next_id.lock().map_err(|e| e.to_string())?;
        *next_id += 1;
        account.id = *next_id;
        accounts.insert(account.account_id.clone(), account.clone());
        Ok(account)
    }

    fn update_account(&self, account: &XianyuAccount) -> OpenDeskResult<()> {
        let mut accounts = self.accounts.lock().map_err(|e| e.to_string())?;
        if !accounts.contains_key(&account.account_id) {
            return Err(format!("账号 {} 不存在", account.account_id).into());
        }
        accounts.insert(account.account_id.clone(), account.clone());
        Ok(())
    }

    fn delete_account(&self, owner_id: i64, account_id: &str) -> OpenDeskResult<()> {
        let mut accounts = self.accounts.lock().map_err(|e| e.to_string())?;
        let owned = accounts
            .get(account_id)
            .map(|account| account.owner_id == owner_id)
            .unwrap_or(false);
        if owned {
            accounts.remove(account_id);
        }
        Ok(())
    }

    fn find_by_account_id(&self, account_id: &str) -> OpenDeskResult<Option<XianyuAccount>> {
        let accounts = self.accounts.lock().map_err(|e| e.to_string())?;
        Ok(accounts.get(account_id).cloned())
    }
}

/// 应用状态（内存存储，与壳层同构；后续可换 SQLite）。
#[derive(Clone)]
pub struct AppState {
    store: Arc<InMemoryAccountStore>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            store: Arc::new(InMemoryAccountStore::new()),
        }
    }

    /// 访问账号存储（实现 `AccountStore`）。
    pub fn store(&self) -> &dyn AccountStore {
        self.store.as_ref()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
