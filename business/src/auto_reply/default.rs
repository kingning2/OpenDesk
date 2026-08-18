//! 默认回复 — 兜底策略，支持"只回复一次"。
//!
//! 对齐 Python 版 get_default_reply：
//! - 未命中关键词且 AI 未启用/未生成时使用默认回复；
//! - 账号级默认回复（可选仅回复一次）；商品级默认回复优先。

use std::collections::HashSet;

/// 默认回复存储 trait — 业务层从存储加载。
pub trait DefaultReplyStore: Send + Sync {
    /// 取默认回复：账号级 + 商品级（商品级优先）。
    /// 返回 `(内容, 是否仅回复一次)`。
    fn default_reply(&self, account_id: &str, item_id: Option<&str>) -> Option<(String, bool)>;

    /// 查询该用户+商品是否已回复过（仅回复一次场景）。
    fn has_replied(&self, account_id: &str, user_id: &str, item_id: Option<&str>) -> bool;

    /// 标记已回复。
    fn mark_replied(&self, account_id: &str, user_id: &str, item_id: Option<&str>);
}

/// 内存实现（开发/测试用；生产由业务层注入存储实现）。
#[derive(Default)]
pub struct InMemoryDefaultReply {
    replies: Vec<(String, String, String, String, bool)>, // (account, item, content, once)
    replied: std::sync::Mutex<HashSet<String>>,
}

fn reply_key(account_id: &str, user_id: &str, item_id: Option<&str>) -> String {
    format!("{account_id}|{user_id}|{}", item_id.unwrap_or(""))
}

impl InMemoryDefaultReply {
    pub fn add(&mut self, account_id: &str, item_id: &str, content: &str, once: bool) {
        self.replies.push((
            account_id.to_string(),
            item_id.to_string(),
            content.to_string(),
            String::new(),
            once,
        ));
    }
}

impl DefaultReplyStore for InMemoryDefaultReply {
    fn default_reply(&self, account_id: &str, item_id: Option<&str>) -> Option<(String, bool)> {
        // 商品级优先，其次账号级（item_id 为空的规则）。
        self.replies
            .iter()
            .filter(|(acc, item, _, _, _)| acc == account_id && !item.is_empty())
            .find(|(_, item, _, _, _)| item_id.is_some_and(|id| item == id))
            .map(|(_, _, content, _, once)| (content.clone(), *once))
            .or_else(|| {
                self.replies
                    .iter()
                    .find(|(acc, item, _, _, _)| acc == account_id && item.is_empty())
                    .map(|(_, _, content, _, once)| (content.clone(), *once))
            })
    }

    fn has_replied(&self, account_id: &str, user_id: &str, item_id: Option<&str>) -> bool {
        self.replied
            .lock()
            .map(|set| set.contains(&reply_key(account_id, user_id, item_id)))
            .unwrap_or(false)
    }

    fn mark_replied(&self, account_id: &str, user_id: &str, item_id: Option<&str>) {
        if let Ok(mut set) = self.replied.lock() {
            set.insert(reply_key(account_id, user_id, item_id));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_reply_preferred_over_account() {
        let mut store = InMemoryDefaultReply::default();
        store.add("acc-1", "", "账号默认回复", false);
        store.add("acc-1", "item-1", "商品默认回复", false);
        let reply = store.default_reply("acc-1", Some("item-1")).expect("reply");
        assert_eq!(reply.0, "商品默认回复");
    }

    #[test]
    fn account_reply_fallback() {
        let mut store = InMemoryDefaultReply::default();
        store.add("acc-1", "", "账号默认回复", false);
        let reply = store.default_reply("acc-1", Some("item-9")).expect("reply");
        assert_eq!(reply.0, "账号默认回复");
    }

    #[test]
    fn once_reply_tracks_user() {
        let mut store = InMemoryDefaultReply::default();
        store.add("acc-1", "", "默认回复", true);
        assert!(!store.has_replied("acc-1", "user-1", None));
        store.mark_replied("acc-1", "user-1", None);
        assert!(store.has_replied("acc-1", "user-1", None));
        // 不同用户不受影响。
        assert!(!store.has_replied("acc-1", "user-2", None));
    }

    #[test]
    fn no_reply_configured() {
        let store = InMemoryDefaultReply::default();
        assert!(store.default_reply("acc-1", None).is_none());
    }
}
