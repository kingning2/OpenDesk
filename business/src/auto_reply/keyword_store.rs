//! 关键词规则存储与服务 — 按账号整表替换 / 增删查。
//!
//! 对齐 Python 版 `keywords-with-item-id` 语义：
//! - 按账号查询全部关键词（含 item_id / type）；
//! - 整表替换（保存时全量覆盖）；
//! - 新增前查重（keyword + item_id 组合）。

use super::keyword::KeywordRule;
use common::DingDaResult;

/// 关键词存储 Port。
pub trait KeywordStore: Send + Sync {
    /// 按账号查询全部关键词。
    fn list_keywords(&self, account_id: &str) -> DingDaResult<Vec<KeywordRule>>;

    /// 按账号整表替换。
    fn replace_keywords(&self, account_id: &str, rules: &[KeywordRule]) -> DingDaResult<()>;

    /// 新增关键词（返回带 id 的规则）。
    fn add_keyword(&self, rule: &KeywordRule) -> DingDaResult<KeywordRule>;

    /// 删除关键词。
    fn delete_keyword(&self, rule_id: i64) -> DingDaResult<()>;
}

/// 关键词服务。
pub struct KeywordService<'a> {
    store: &'a dyn KeywordStore,
}

impl<'a> KeywordService<'a> {
    pub fn new(store: &'a dyn KeywordStore) -> Self {
        Self { store }
    }

    /// 按账号查询。
    pub fn list(&self, account_id: &str) -> DingDaResult<Vec<KeywordRule>> {
        self.store.list_keywords(account_id)
    }

    /// 整表替换（保存）。
    pub fn replace(&self, account_id: &str, rules: &[KeywordRule]) -> DingDaResult<()> {
        self.store.replace_keywords(account_id, rules)
    }

    /// 新增（keyword + item_id 组合查重）。
    pub fn add(&self, account_id: &str, mut rule: KeywordRule) -> DingDaResult<KeywordRule> {
        rule.account_id = account_id.to_string();
        rule.keyword = rule.keyword.trim().to_string();
        if rule.keyword.is_empty() {
            return Err("关键词不能为空".to_string().into());
        }
        let existing = self.store.list_keywords(account_id)?;
        let duplicate = existing
            .iter()
            .any(|r| r.keyword == rule.keyword && (r.item_id == rule.item_id));
        if duplicate {
            return Err("该关键词已存在（相同商品）".to_string().into());
        }
        self.store.add_keyword(&rule)
    }

    /// 删除。
    pub fn delete(&self, rule_id: i64) -> DingDaResult<()> {
        self.store.delete_keyword(rule_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockStore {
        rules: Mutex<Vec<KeywordRule>>,
        next_id: Mutex<i64>,
    }

    impl MockStore {
        fn new(rules: Vec<KeywordRule>) -> Self {
            let len = rules.len();
            Self {
                rules: Mutex::new(rules),
                next_id: Mutex::new(len as i64),
            }
        }
    }

    impl KeywordStore for MockStore {
        fn list_keywords(&self, account_id: &str) -> DingDaResult<Vec<KeywordRule>> {
            Ok(self
                .rules
                .lock()
                .expect("lock")
                .iter()
                .filter(|r| r.account_id == account_id)
                .cloned()
                .collect())
        }
        fn replace_keywords(&self, account_id: &str, rules: &[KeywordRule]) -> DingDaResult<()> {
            let mut all = self.rules.lock().expect("lock");
            all.retain(|r| r.account_id != account_id);
            for rule in rules {
                let mut rule = rule.clone();
                rule.account_id = account_id.to_string();
                all.push(rule);
            }
            Ok(())
        }
        fn add_keyword(&self, rule: &KeywordRule) -> DingDaResult<KeywordRule> {
            let mut rule = rule.clone();
            let mut next = self.next_id.lock().expect("lock");
            *next += 1;
            rule.id = *next;
            self.rules.lock().expect("lock").push(rule.clone());
            Ok(rule)
        }
        fn delete_keyword(&self, rule_id: i64) -> DingDaResult<()> {
            self.rules.lock().expect("lock").retain(|r| r.id != rule_id);
            Ok(())
        }
    }

    fn rule(keyword: &str, item_id: &str) -> KeywordRule {
        KeywordRule {
            id: 0,
            account_id: "acc-1".to_string(),
            keyword: keyword.to_string(),
            reply: "回复".to_string(),
            item_id: item_id.to_string(),
            rule_type: "text".to_string(),
            image_url: String::new(),
            item_title: String::new(),
        }
    }

    #[test]
    fn list_filters_by_account() {
        let store = MockStore::new(vec![rule("在吗", ""), rule("价格", "")]);
        let service = KeywordService::new(&store);
        let list = service.list("acc-1").expect("list");
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn add_rejects_duplicate() {
        let store = MockStore::new(vec![rule("在吗", "item-1")]);
        let service = KeywordService::new(&store);
        assert!(service.add("acc-1", rule("在吗", "item-1")).is_err());
        assert!(service.add("acc-1", rule("在吗", "item-2")).is_ok());
    }

    #[test]
    fn add_rejects_empty_keyword() {
        let store = MockStore::new(vec![]);
        let service = KeywordService::new(&store);
        assert!(service.add("acc-1", rule("  ", "")).is_err());
    }

    #[test]
    fn replace_overwrites_account_rules() {
        let store = MockStore::new(vec![rule("旧关键词", "")]);
        let service = KeywordService::new(&store);
        service
            .replace("acc-1", &[rule("新关键词", "")])
            .expect("replace");
        let list = service.list("acc-1").expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].keyword, "新关键词");
    }
}
