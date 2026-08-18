//! 返佣规则服务 — 校验 + CRUD 编排。
//!
//! 对齐 Python 版 `product_rule_service` / `publish_rule_service` 的业务规则：
//! - 选品规则：类目/关键词至少一项；账号归属 + 启用校验；daily_count >= 1；
//! - 发布规则：同一账号仅一条；daily_count >= 1。

use super::rule::{ProductRule, ProductRuleInput, PublishRule, PublishRuleInput, RuleStatus};
use super::store::{PublishRuleStore, RuleStore};
use thiserror::Error;

/// 规则服务错误。
#[derive(Debug, Error)]
pub enum RuleServiceError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("store error: {0}")]
    Store(String),
    #[error("not found: rule {0}")]
    NotFound(i64),
}

/// 校验错误（供前端展示）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub message: String,
}

/// 选品规则服务。
pub struct ProductRuleService<'a> {
    store: &'a dyn RuleStore,
}

impl<'a> ProductRuleService<'a> {
    pub fn new(store: &'a dyn RuleStore) -> Self {
        Self { store }
    }

    /// 校验规则输入（不落库）。
    pub fn validate(&self, input: &ProductRuleInput) -> Result<(), ValidationError> {
        let cat = input.cat.as_deref().map(str::trim).unwrap_or("");
        let keyword = input.keyword.as_deref().map(str::trim).unwrap_or("");
        if cat.is_empty() && keyword.is_empty() {
            return Err(ValidationError {
                message: "商品类目和关键词至少填写一项".to_string(),
            });
        }
        if input.account_id.trim().is_empty() {
            return Err(ValidationError {
                message: "请选择闲鱼账号".to_string(),
            });
        }
        Ok(())
    }

    /// 校验账号归属与状态。
    fn validate_account(&self, owner_id: i64, account_id: &str) -> Result<(), ValidationError> {
        let check = self
            .store
            .check_account(owner_id, account_id)
            .map_err(|error| ValidationError {
                message: error.to_string(),
            })?;
        if !check.exists {
            return Err(ValidationError {
                message: "所选闲鱼账号不存在或不属于当前用户".to_string(),
            });
        }
        if !check.active {
            return Err(ValidationError {
                message: "所选闲鱼账号未启用，请先启用账号后再保存规则".to_string(),
            });
        }
        Ok(())
    }

    /// 分页查询。
    pub fn list(
        &self,
        owner_id: i64,
        page: u32,
        page_size: u32,
        is_admin: bool,
    ) -> Result<(Vec<ProductRule>, u32), RuleServiceError> {
        self.store
            .list_product_rules(owner_id, page, page_size, is_admin)
            .map_err(|e| RuleServiceError::Store(e.to_string()))
    }

    /// 新建规则（校验后落库）。
    pub fn create(&self, input: &ProductRuleInput) -> Result<ProductRule, RuleServiceError> {
        self.validate(input)
            .map_err(|error| RuleServiceError::Validation(error.message))?;
        self.validate_account(input.owner_id, &input.account_id)
            .map_err(|error| RuleServiceError::Validation(error.message))?;

        let rule = ProductRule {
            id: 0,
            owner_id: input.owner_id,
            account_id: input.account_id.clone(),
            rule_name: if input.rule_name.trim().is_empty() {
                "未命名规则".to_string()
            } else {
                input.rule_name.clone()
            },
            cat: input
                .cat
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from),
            cat_name: input.cat_name.clone(),
            keyword: input
                .keyword
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from),
            sort: if input.sort.trim().is_empty() {
                "default".to_string()
            } else {
                input.sort.clone()
            },
            daily_count: input.daily_count.max(1),
            status: RuleStatus::from_bool(input.enabled),
            remark: input.remark.clone(),
        };
        self.store
            .create_product_rule(&rule)
            .map_err(|e| RuleServiceError::Store(e.to_string()))
    }

    /// 更新规则。
    pub fn update(&self, rule: &ProductRule) -> Result<(), RuleServiceError> {
        if !rule.has_source() {
            return Err(RuleServiceError::Validation(
                "商品类目和关键词至少填写一项".to_string(),
            ));
        }
        self.store
            .update_product_rule(rule)
            .map_err(|e| RuleServiceError::Store(e.to_string()))
    }

    /// 删除规则。
    pub fn delete(&self, rule_id: i64) -> Result<(), RuleServiceError> {
        self.store
            .delete_product_rule(rule_id)
            .map_err(|e| RuleServiceError::Store(e.to_string()))
    }

    /// 切换启用状态。
    pub fn toggle(&self, rule_id: i64, enabled: bool) -> Result<(), RuleServiceError> {
        self.store
            .toggle_product_rule(rule_id, enabled)
            .map_err(|e| RuleServiceError::Store(e.to_string()))
    }
}

/// 发布规则服务。
pub struct PublishRuleService<'a> {
    store: &'a dyn PublishRuleStore,
    /// 账号校验复用选品规则的存储（同一账号表）。
    account_store: &'a dyn RuleStore,
}

impl<'a> PublishRuleService<'a> {
    pub fn new(store: &'a dyn PublishRuleStore, account_store: &'a dyn RuleStore) -> Self {
        Self {
            store,
            account_store,
        }
    }

    /// 分页查询。
    pub fn list(
        &self,
        owner_id: i64,
        page: u32,
        page_size: u32,
        is_admin: bool,
    ) -> Result<(Vec<PublishRule>, u32), RuleServiceError> {
        self.store
            .list_publish_rules(owner_id, page, page_size, is_admin)
            .map_err(|e| RuleServiceError::Store(e.to_string()))
    }

    /// 新建发布规则：同一账号仅一条 + 账号归属校验。
    pub fn create(&self, input: &PublishRuleInput) -> Result<PublishRule, RuleServiceError> {
        if input.account_id.trim().is_empty() {
            return Err(RuleServiceError::Validation("请选择闲鱼账号".to_string()));
        }
        let check = self
            .account_store
            .check_account(input.owner_id, &input.account_id)
            .map_err(|e| RuleServiceError::Store(e.to_string()))?;
        if !check.exists {
            return Err(RuleServiceError::Validation(
                "所选闲鱼账号不存在或不属于当前用户".to_string(),
            ));
        }
        if !check.active {
            return Err(RuleServiceError::Validation(
                "所选闲鱼账号未启用，请先启用账号后再保存规则".to_string(),
            ));
        }

        // 同一账号唯一性。
        if let Some(_existing) = self
            .store
            .get_publish_rule_by_account(input.owner_id, &input.account_id)
            .map_err(|e| RuleServiceError::Store(e.to_string()))?
        {
            return Err(RuleServiceError::Validation(
                "同一闲鱼账号只允许创建一条发布规则，请直接编辑现有规则".to_string(),
            ));
        }

        let rule = PublishRule {
            id: 0,
            owner_id: input.owner_id,
            rule_name: if input.rule_name.trim().is_empty() {
                "未命名发布规则".to_string()
            } else {
                input.rule_name.clone()
            },
            account_id: input.account_id.clone(),
            daily_count: input.daily_count.max(1),
            status: RuleStatus::from_bool(input.enabled),
            remark: input.remark.clone(),
        };
        self.store
            .create_publish_rule(&rule)
            .map_err(|e| RuleServiceError::Store(e.to_string()))
    }

    /// 更新发布规则。
    pub fn update(&self, rule: &PublishRule) -> Result<(), RuleServiceError> {
        self.store
            .update_publish_rule(rule)
            .map_err(|e| RuleServiceError::Store(e.to_string()))
    }

    /// 删除发布规则。
    pub fn delete(&self, rule_id: i64) -> Result<(), RuleServiceError> {
        self.store
            .delete_publish_rule(rule_id)
            .map_err(|e| RuleServiceError::Store(e.to_string()))
    }

    /// 切换启用状态。
    pub fn toggle(&self, rule_id: i64, enabled: bool) -> Result<(), RuleServiceError> {
        self.store
            .toggle_publish_rule(rule_id, enabled)
            .map_err(|e| RuleServiceError::Store(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::promotion::store::AccountCheck;
    use common::DingDaResult;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockRuleStore {
        accounts: HashMap<String, AccountCheck>,
        rules: Mutex<Vec<ProductRule>>,
        publish: Mutex<Vec<PublishRule>>,
    }

    impl MockRuleStore {
        fn new(account_active: bool) -> Self {
            let mut accounts = HashMap::new();
            accounts.insert(
                "acc-1".to_string(),
                AccountCheck {
                    exists: true,
                    active: account_active,
                },
            );
            Self {
                accounts,
                rules: Mutex::new(vec![]),
                publish: Mutex::new(vec![]),
            }
        }
    }

    impl RuleStore for MockRuleStore {
        fn list_product_rules(
            &self,
            _owner_id: i64,
            _page: u32,
            _page_size: u32,
            _is_admin: bool,
        ) -> DingDaResult<(Vec<ProductRule>, u32)> {
            let rules = self.rules.lock().expect("rules lock").clone();
            Ok((rules.clone(), rules.len() as u32))
        }
        fn create_product_rule(&self, rule: &ProductRule) -> DingDaResult<ProductRule> {
            let mut rule = rule.clone();
            rule.id = 1;
            self.rules.lock().expect("rules lock").push(rule.clone());
            Ok(rule)
        }
        fn update_product_rule(&self, _rule: &ProductRule) -> DingDaResult<()> {
            Ok(())
        }
        fn delete_product_rule(&self, _rule_id: i64) -> DingDaResult<()> {
            Ok(())
        }
        fn toggle_product_rule(&self, _rule_id: i64, _enabled: bool) -> DingDaResult<()> {
            Ok(())
        }
        fn get_product_rule(&self, _rule_id: i64) -> DingDaResult<Option<ProductRule>> {
            Ok(None)
        }
        fn check_account(&self, _owner_id: i64, account_id: &str) -> DingDaResult<AccountCheck> {
            Ok(self
                .accounts
                .get(account_id)
                .cloned()
                .unwrap_or(AccountCheck {
                    exists: false,
                    active: false,
                }))
        }
    }

    impl PublishRuleStore for MockRuleStore {
        fn list_publish_rules(
            &self,
            _owner_id: i64,
            _page: u32,
            _page_size: u32,
            _is_admin: bool,
        ) -> DingDaResult<(Vec<PublishRule>, u32)> {
            let rules = self.publish.lock().expect("publish lock").clone();
            Ok((rules.clone(), rules.len() as u32))
        }
        fn get_publish_rule_by_account(
            &self,
            _owner_id: i64,
            account_id: &str,
        ) -> DingDaResult<Option<PublishRule>> {
            let found = self
                .publish
                .lock()
                .expect("publish lock")
                .iter()
                .find(|r| r.account_id == account_id)
                .cloned();
            Ok(found)
        }
        fn create_publish_rule(&self, rule: &PublishRule) -> DingDaResult<PublishRule> {
            let mut rule = rule.clone();
            rule.id = 1;
            self.publish
                .lock()
                .expect("publish lock")
                .push(rule.clone());
            Ok(rule)
        }
        fn update_publish_rule(&self, _rule: &PublishRule) -> DingDaResult<()> {
            Ok(())
        }
        fn delete_publish_rule(&self, _rule_id: i64) -> DingDaResult<()> {
            Ok(())
        }
        fn toggle_publish_rule(&self, _rule_id: i64, _enabled: bool) -> DingDaResult<()> {
            Ok(())
        }
    }

    fn product_input(owner: i64, cat: Option<&str>, keyword: Option<&str>) -> ProductRuleInput {
        ProductRuleInput {
            owner_id: owner,
            account_id: "acc-1".to_string(),
            rule_name: "手机选品".to_string(),
            cat: cat.map(String::from),
            cat_name: None,
            keyword: keyword.map(String::from),
            sort: "default".to_string(),
            daily_count: 10,
            enabled: true,
            remark: None,
        }
    }

    #[test]
    fn product_rule_rejects_empty_source() {
        let store = MockRuleStore::new(true);
        let service = ProductRuleService::new(&store);
        let input = product_input(1, None, None);
        assert!(matches!(
            service.create(&input),
            Err(RuleServiceError::Validation(_))
        ));
    }

    #[test]
    fn product_rule_rejects_inactive_account() {
        let store = MockRuleStore::new(false);
        let service = ProductRuleService::new(&store);
        let input = product_input(1, Some("手机"), None);
        assert!(matches!(
            service.create(&input),
            Err(RuleServiceError::Validation(_))
        ));
    }

    #[test]
    fn product_rule_creates_with_cat() {
        let store = MockRuleStore::new(true);
        let service = ProductRuleService::new(&store);
        let input = product_input(1, Some("手机"), None);
        let rule = service.create(&input).expect("create");
        assert_eq!(rule.rule_name, "手机选品");
        assert_eq!(rule.daily_count, 10);
        assert_eq!(rule.status, RuleStatus::Enabled);
    }

    #[test]
    fn product_rule_creates_with_keyword() {
        let store = MockRuleStore::new(true);
        let service = ProductRuleService::new(&store);
        let input = product_input(1, None, Some("耳机"));
        let rule = service.create(&input).expect("create");
        assert_eq!(rule.keyword.as_deref(), Some("耳机"));
    }

    #[test]
    fn product_rule_normalizes_name_and_count() {
        let store = MockRuleStore::new(true);
        let service = ProductRuleService::new(&store);
        let mut input = product_input(1, Some("手机"), None);
        input.rule_name = "  ".to_string();
        input.daily_count = 0;
        let rule = service.create(&input).expect("create");
        assert_eq!(rule.rule_name, "未命名规则");
        assert_eq!(rule.daily_count, 1);
    }

    #[test]
    fn publish_rule_rejects_duplicate_account() {
        let store = MockRuleStore::new(true);
        let service = PublishRuleService::new(&store, &store);
        let input = PublishRuleInput {
            owner_id: 1,
            rule_name: "发布".to_string(),
            account_id: "acc-1".to_string(),
            daily_count: 5,
            enabled: true,
            remark: None,
        };
        let first = service.create(&input).expect("first create");
        assert_eq!(first.account_id, "acc-1");
        assert!(matches!(
            service.create(&input),
            Err(RuleServiceError::Validation(_))
        ));
    }

    #[test]
    fn publish_rule_requires_account() {
        let store = MockRuleStore::new(true);
        let service = PublishRuleService::new(&store, &store);
        let input = PublishRuleInput {
            owner_id: 1,
            rule_name: "发布".to_string(),
            account_id: "  ".to_string(),
            daily_count: 5,
            enabled: true,
            remark: None,
        };
        assert!(matches!(
            service.create(&input),
            Err(RuleServiceError::Validation(_))
        ));
    }
}
