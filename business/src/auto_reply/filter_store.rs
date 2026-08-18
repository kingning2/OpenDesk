//! 消息过滤规则存储与服务 — 过滤规则 CRUD。
//!
//! 对齐 Python 版 `/api/v1/message-filters` 语义：
//! - 按账号查询过滤规则（skip_reply / skip_notify 两类）；
//! - 新建 / 更新 / 删除 / 启用切换。

use super::filter::FilterRule;
use common::DingDaResult;

/// 过滤规则存储 Port。
pub trait FilterStore: Send + Sync {
    /// 按账号查询全部过滤规则。
    fn list_filters(&self, owner_id: i64, account_id: &str) -> DingDaResult<Vec<FilterRule>>;

    /// 新建。
    fn create_filter(&self, rule: &FilterRule) -> DingDaResult<FilterRule>;

    /// 更新。
    fn update_filter(&self, owner_id: i64, rule: &FilterRule) -> DingDaResult<()>;

    /// 删除。
    fn delete_filter(&self, owner_id: i64, rule_id: i64) -> DingDaResult<()>;

    /// 切换启用状态。
    fn set_enabled(&self, owner_id: i64, rule_id: i64, enabled: bool) -> DingDaResult<()>;
}

/// 过滤规则服务。
pub struct FilterService<'a> {
    store: &'a dyn FilterStore,
}

impl<'a> FilterService<'a> {
    pub fn new(store: &'a dyn FilterStore) -> Self {
        Self { store }
    }

    /// 按账号查询。
    pub fn list(&self, owner_id: i64, account_id: &str) -> DingDaResult<Vec<FilterRule>> {
        self.store.list_filters(owner_id, account_id)
    }

    /// 新建（关键词必填）。
    pub fn create(
        &self,
        owner_id: i64,
        account_id: &str,
        mut rule: FilterRule,
    ) -> DingDaResult<FilterRule> {
        rule.owner_id = owner_id;
        rule.account_id = account_id.to_string();
        rule.keyword = rule.keyword.trim().to_string();
        if rule.keyword.is_empty() {
            return Err("过滤关键词不能为空".to_string().into());
        }
        self.store.create_filter(&rule)
    }

    /// 更新。
    pub fn update(&self, owner_id: i64, rule: &FilterRule) -> DingDaResult<()> {
        if rule.keyword.trim().is_empty() {
            return Err("过滤关键词不能为空".to_string().into());
        }
        self.store.update_filter(owner_id, rule)
    }

    /// 删除。
    pub fn delete(&self, owner_id: i64, rule_id: i64) -> DingDaResult<()> {
        self.store.delete_filter(owner_id, rule_id)
    }

    /// 切换启用状态。
    pub fn set_enabled(&self, owner_id: i64, rule_id: i64, enabled: bool) -> DingDaResult<()> {
        self.store.set_enabled(owner_id, rule_id, enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auto_reply::filter::FilterType;
    use std::sync::Mutex;

    struct MockStore {
        rules: Mutex<Vec<FilterRule>>,
        next_id: Mutex<i64>,
    }

    impl MockStore {
        fn new(rules: Vec<FilterRule>) -> Self {
            let len = rules.len() as i64;
            Self {
                rules: Mutex::new(rules),
                next_id: Mutex::new(len),
            }
        }
    }

    impl FilterStore for MockStore {
        fn list_filters(&self, owner_id: i64, account_id: &str) -> DingDaResult<Vec<FilterRule>> {
            Ok(self
                .rules
                .lock()
                .expect("lock")
                .iter()
                .filter(|r| r.owner_id == owner_id && r.account_id == account_id)
                .cloned()
                .collect())
        }
        fn create_filter(&self, rule: &FilterRule) -> DingDaResult<FilterRule> {
            let mut rule = rule.clone();
            let mut next = self.next_id.lock().expect("lock");
            *next += 1;
            rule.id = *next;
            self.rules.lock().expect("lock").push(rule.clone());
            Ok(rule)
        }
        fn update_filter(&self, owner_id: i64, rule: &FilterRule) -> DingDaResult<()> {
            let mut list = self.rules.lock().expect("lock");
            let Some(existing) = list
                .iter_mut()
                .find(|r| r.id == rule.id && r.owner_id == owner_id)
            else {
                return Err("不存在或无权限".to_string().into());
            };
            *existing = rule.clone();
            existing.owner_id = owner_id;
            Ok(())
        }
        fn delete_filter(&self, owner_id: i64, rule_id: i64) -> DingDaResult<()> {
            let mut list = self.rules.lock().expect("lock");
            let before = list.len();
            list.retain(|r| !(r.id == rule_id && r.owner_id == owner_id));
            if list.len() == before {
                return Err("不存在或无权限".to_string().into());
            }
            Ok(())
        }
        fn set_enabled(&self, owner_id: i64, rule_id: i64, enabled: bool) -> DingDaResult<()> {
            let mut list = self.rules.lock().expect("lock");
            let Some(rule) = list
                .iter_mut()
                .find(|r| r.id == rule_id && r.owner_id == owner_id)
            else {
                return Err("不存在或无权限".to_string().into());
            };
            rule.enabled = enabled;
            Ok(())
        }
    }

    fn rule(keyword: &str, filter_type: FilterType) -> FilterRule {
        FilterRule {
            id: 0,
            account_id: String::new(),
            owner_id: 0,
            filter_type,
            keyword: keyword.to_string(),
            enabled: true,
        }
    }

    #[test]
    fn create_requires_keyword() {
        let store = MockStore::new(vec![]);
        let service = FilterService::new(&store);
        assert!(service
            .create(1, "acc-1", rule("  ", FilterType::SkipReply))
            .is_err());
        assert!(service
            .create(1, "acc-1", rule("勿扰", FilterType::SkipReply))
            .is_ok());
    }

    #[test]
    fn list_filters_by_account() {
        let store = MockStore::new(vec![]);
        let service = FilterService::new(&store);
        service
            .create(1, "acc-1", rule("勿扰", FilterType::SkipReply))
            .expect("create");
        service
            .create(1, "acc-2", rule("广告", FilterType::SkipNotify))
            .expect("create");
        assert_eq!(service.list(1, "acc-1").expect("list").len(), 1);
    }

    #[test]
    fn set_enabled_respects_ownership() {
        let store = MockStore::new(vec![]);
        let service = FilterService::new(&store);
        let created = service
            .create(1, "acc-1", rule("勿扰", FilterType::SkipReply))
            .expect("create");
        assert!(service.set_enabled(2, created.id, false).is_err());
        assert!(service.set_enabled(1, created.id, false).is_ok());
    }

    #[test]
    fn update_respects_ownership() {
        let store = MockStore::new(vec![]);
        let service = FilterService::new(&store);
        let created = service
            .create(1, "acc-1", rule("勿扰", FilterType::SkipReply))
            .expect("create");
        let mut other_owner = created.clone();
        other_owner.keyword = "篡改".to_string();
        assert!(service.update(2, &other_owner).is_err());
        let mut valid = created;
        valid.keyword = "新词".to_string();
        assert!(service.update(1, &valid).is_ok());
    }

    #[test]
    fn delete_respects_ownership() {
        let store = MockStore::new(vec![]);
        let service = FilterService::new(&store);
        let created = service
            .create(1, "acc-1", rule("勿扰", FilterType::SkipReply))
            .expect("create");
        assert!(service.delete(2, created.id).is_err());
        assert!(service.delete(1, created.id).is_ok());
    }
}
