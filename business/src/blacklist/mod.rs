//! 黑名单管理 — 个人黑名单 CRUD + 平台黑名单查询。
//!
//! 对齐 Python 版黑名单管理业务（`/api/v1/blacklist`）：
//! - 个人黑名单：buyer_id 必填，支持商品级（account+item）/ 账户级（account）/ 用户级（全空）三级；
//! - 新建（批量 buyer_ids）、查询、启用切换、删除、批量删除；
//! - 平台黑名单：查询列表。

use common::OpenDeskResult;
use serde::{Deserialize, Serialize};

/// 个人黑名单条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalBlacklistItem {
    pub id: i64,
    pub owner_id: i64,
    pub account_id: Option<String>,
    pub buyer_id: String,
    pub buyer_nick: Option<String>,
    pub item_id: Option<String>,
    pub reason: Option<String>,
    pub is_enabled: bool,
    pub created_at: Option<String>,
}

impl PersonalBlacklistItem {
    /// 命中级别（对齐 delivery 规则引擎三级匹配）。
    pub fn level(&self) -> &'static str {
        match (&self.account_id, &self.item_id) {
            (Some(_), Some(_)) => "商品级",
            (Some(_), None) => "账户级",
            _ => "用户级",
        }
    }
}

/// 平台黑名单条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformBlacklistItem {
    pub id: i64,
    pub owner_id: i64,
    pub buyer_id: String,
    pub buyer_nick: Option<String>,
    pub created_at: Option<String>,
}

/// 个人黑名单查询条件。
#[derive(Debug, Clone, Default)]
pub struct BlacklistQuery {
    pub page: u32,
    pub page_size: u32,
    pub buyer_id: String,
    pub buyer_nick: String,
}

/// 黑名单存储 Port。
pub trait BlacklistStore: Send + Sync {
    /// 分页查询个人黑名单。
    fn list_personal(
        &self,
        owner_id: i64,
        query: &BlacklistQuery,
    ) -> OpenDeskResult<(Vec<PersonalBlacklistItem>, u32)>;

    /// 分页查询平台黑名单。
    fn list_platform(
        &self,
        owner_id: i64,
        query: &BlacklistQuery,
    ) -> OpenDeskResult<(Vec<PlatformBlacklistItem>, u32)>;

    /// 新建个人黑名单条目。
    fn create_personal(
        &self,
        item: &PersonalBlacklistItem,
    ) -> OpenDeskResult<PersonalBlacklistItem>;

    /// 切换启用状态。
    fn set_enabled(&self, owner_id: i64, id: i64, enabled: bool) -> OpenDeskResult<()>;

    /// 删除。
    fn delete(&self, owner_id: i64, id: i64) -> OpenDeskResult<()>;
}

/// 黑名单服务。
pub struct BlacklistService<'a> {
    store: &'a dyn BlacklistStore,
}

impl<'a> BlacklistService<'a> {
    pub fn new(store: &'a dyn BlacklistStore) -> Self {
        Self { store }
    }

    /// 分页查询个人黑名单。
    pub fn list_personal(
        &self,
        owner_id: i64,
        query: &BlacklistQuery,
    ) -> OpenDeskResult<(Vec<PersonalBlacklistItem>, u32)> {
        self.store.list_personal(owner_id, query)
    }

    /// 分页查询平台黑名单。
    pub fn list_platform(
        &self,
        owner_id: i64,
        query: &BlacklistQuery,
    ) -> OpenDeskResult<(Vec<PlatformBlacklistItem>, u32)> {
        self.store.list_platform(owner_id, query)
    }

    /// 新建（buyer_id 必填）。
    pub fn create(
        &self,
        owner_id: i64,
        buyer_id: &str,
        account_id: Option<&str>,
        item_id: Option<&str>,
        reason: Option<&str>,
    ) -> OpenDeskResult<PersonalBlacklistItem> {
        let buyer_id = buyer_id.trim();
        if buyer_id.is_empty() {
            return Err("买家 ID 不能为空".to_string().into());
        }
        let item = PersonalBlacklistItem {
            id: 0,
            owner_id,
            account_id: account_id
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from),
            buyer_id: buyer_id.to_string(),
            buyer_nick: None,
            item_id: item_id
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from),
            reason: reason
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from),
            is_enabled: true,
            created_at: None,
        };
        self.store.create_personal(&item)
    }

    /// 切换启用状态。
    pub fn set_enabled(&self, owner_id: i64, id: i64, enabled: bool) -> OpenDeskResult<()> {
        self.store.set_enabled(owner_id, id, enabled)
    }

    /// 删除。
    pub fn delete(&self, owner_id: i64, id: i64) -> OpenDeskResult<()> {
        self.store.delete(owner_id, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockStore {
        items: Mutex<Vec<PersonalBlacklistItem>>,
        next_id: Mutex<i64>,
    }

    impl MockStore {
        fn new(items: Vec<PersonalBlacklistItem>) -> Self {
            let len = items.len() as i64;
            Self {
                items: Mutex::new(items),
                next_id: Mutex::new(len),
            }
        }
    }

    impl BlacklistStore for MockStore {
        fn list_personal(
            &self,
            owner_id: i64,
            query: &BlacklistQuery,
        ) -> OpenDeskResult<(Vec<PersonalBlacklistItem>, u32)> {
            let list: Vec<PersonalBlacklistItem> = self
                .items
                .lock()
                .expect("lock")
                .iter()
                .filter(|item| {
                    item.owner_id == owner_id
                        && (query.buyer_id.is_empty() || item.buyer_id.contains(&query.buyer_id))
                        && (query.buyer_nick.is_empty()
                            || item
                                .buyer_nick
                                .as_deref()
                                .is_some_and(|nick| nick.contains(&query.buyer_nick)))
                })
                .cloned()
                .collect();
            let total = list.len() as u32;
            Ok((list, total))
        }
        fn list_platform(
            &self,
            _owner_id: i64,
            _query: &BlacklistQuery,
        ) -> OpenDeskResult<(Vec<PlatformBlacklistItem>, u32)> {
            Ok((Vec::new(), 0))
        }
        fn create_personal(
            &self,
            item: &PersonalBlacklistItem,
        ) -> OpenDeskResult<PersonalBlacklistItem> {
            let mut item = item.clone();
            let mut next = self.next_id.lock().expect("lock");
            *next += 1;
            item.id = *next;
            self.items.lock().expect("lock").push(item.clone());
            Ok(item)
        }
        fn set_enabled(&self, owner_id: i64, id: i64, enabled: bool) -> OpenDeskResult<()> {
            let mut list = self.items.lock().expect("lock");
            let Some(item) = list
                .iter_mut()
                .find(|i| i.id == id && i.owner_id == owner_id)
            else {
                return Err("不存在或无权限".to_string().into());
            };
            item.is_enabled = enabled;
            Ok(())
        }
        fn delete(&self, owner_id: i64, id: i64) -> OpenDeskResult<()> {
            self.items
                .lock()
                .expect("lock")
                .retain(|i| !(i.id == id && i.owner_id == owner_id));
            Ok(())
        }
    }

    fn item(buyer_id: &str, account: Option<&str>, item_id: Option<&str>) -> PersonalBlacklistItem {
        PersonalBlacklistItem {
            id: 0,
            owner_id: 1,
            account_id: account.map(String::from),
            buyer_id: buyer_id.to_string(),
            buyer_nick: None,
            item_id: item_id.map(String::from),
            reason: None,
            is_enabled: true,
            created_at: None,
        }
    }

    #[test]
    fn create_requires_buyer_id() {
        let store = MockStore::new(vec![]);
        let service = BlacklistService::new(&store);
        assert!(service.create(1, "  ", None, None, None).is_err());
        assert!(service.create(1, "buyer-1", None, None, None).is_ok());
    }

    #[test]
    fn level_classification() {
        assert_eq!(item("b", Some("a"), Some("i")).level(), "商品级");
        assert_eq!(item("b", Some("a"), None).level(), "账户级");
        assert_eq!(item("b", None, None).level(), "用户级");
    }

    #[test]
    fn list_filters_by_buyer() {
        let store = MockStore::new(vec![
            item("buyer-1", None, None),
            item("buyer-2", None, None),
        ]);
        let service = BlacklistService::new(&store);
        let query = BlacklistQuery {
            page: 1,
            page_size: 20,
            buyer_id: "buyer-1".to_string(),
            buyer_nick: String::new(),
        };
        assert_eq!(service.list_personal(1, &query).expect("list").1, 1);
    }

    #[test]
    fn set_enabled_respects_ownership() {
        let store = MockStore::new(vec![]);
        let service = BlacklistService::new(&store);
        let created = service
            .create(1, "buyer-1", None, None, None)
            .expect("create");
        assert!(service.set_enabled(2, created.id, false).is_err());
        assert!(service.set_enabled(1, created.id, false).is_ok());
        assert!(service.delete(1, created.id).is_ok());
    }
}
