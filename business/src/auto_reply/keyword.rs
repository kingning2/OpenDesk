//! 关键词匹配 — 商品 ID 优先 / 图片关键词 / 变量替换。
//!
//! 对齐 Python 版 get_keyword_reply：
//! - 关键词支持多行（每行一个触发词）；
//! - 商品 ID 精确匹配优先于全局匹配；
//! - 图片关键词返回图片发送指令（`__IMAGE_SEND__` 协议，由发送层解析）；
//! - 回复支持 `{send_user_name}` / `{send_user_id}` / `{send_message}` / `{item_id}` 变量替换。

use serde::{Deserialize, Serialize};

/// 关键词规则（业务层从存储加载）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordRule {
    /// 存储主键（新建时为 0）。
    #[serde(default)]
    pub id: i64,
    /// 所属账号 ID。
    #[serde(default)]
    pub account_id: String,
    /// 多行关键词，每行一个触发词。
    pub keyword: String,
    /// 回复内容（图片关键词时为图片 URL 或发送指令）。
    pub reply: String,
    /// 关联商品 ID（空表示全局规则）。
    #[serde(default)]
    pub item_id: String,
    /// 关键词类型：text / image。
    #[serde(default = "default_rule_type")]
    pub rule_type: String,
    /// 图片 URL（图片关键词）。
    #[serde(default)]
    pub image_url: String,
    /// 商品标题（日志用）。
    #[serde(default)]
    pub item_title: String,
}

fn default_rule_type() -> String {
    "text".to_string()
}

/// 匹配结果。
#[derive(Debug, Clone)]
pub struct KeywordMatch {
    pub matched_keyword: String,
    pub rule: KeywordRule,
}

/// 关键词匹配器 — 规则集合判定。
pub struct KeywordMatcher {
    rules: Vec<KeywordRule>,
}

/// 图片发送指令前缀（发送层解析：`__IMAGE_SEND__|KW:keyword|url`）。
pub const IMAGE_SEND_PREFIX: &str = "__IMAGE_SEND__";

impl KeywordMatcher {
    pub fn new(rules: Vec<KeywordRule>) -> Self {
        Self { rules }
    }

    /// 匹配关键词。`item_id` 存在时优先匹配商品专属规则。
    pub fn match_message(&self, message: &str, item_id: Option<&str>) -> Option<KeywordMatch> {
        let msg_lower = message.to_lowercase();

        // 1. 商品 ID 精确匹配优先。
        if let Some(item_id) = item_id {
            if let Some(found) = self.match_with_item(&msg_lower, item_id) {
                return Some(found);
            }
        }

        // 2. 全局规则。
        self.match_global(&msg_lower)
    }

    fn match_with_item(&self, msg_lower: &str, item_id: &str) -> Option<KeywordMatch> {
        self.rules
            .iter()
            .filter(|rule| !rule.item_id.is_empty() && rule.item_id == item_id)
            .find_map(|rule| self.try_match(rule, msg_lower))
    }

    fn match_global(&self, msg_lower: &str) -> Option<KeywordMatch> {
        self.rules
            .iter()
            .filter(|rule| rule.item_id.is_empty())
            .find_map(|rule| self.try_match(rule, msg_lower))
    }

    fn try_match(&self, rule: &KeywordRule, msg_lower: &str) -> Option<KeywordMatch> {
        rule.keyword
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .find(|line| msg_lower.contains(&line.to_lowercase()))
            .map(|matched_keyword| KeywordMatch {
                matched_keyword: matched_keyword.to_string(),
                rule: rule.clone(),
            })
    }

    /// 渲染回复（变量替换 + 图片指令组装）。
    pub fn render_reply(
        matched: &KeywordMatch,
        send_user_name: &str,
        send_user_id: &str,
        send_message: &str,
        item_id: &str,
    ) -> String {
        let rule = &matched.rule;
        if rule.rule_type == "image" && !rule.image_url.is_empty() {
            // 图片关键词：携带规则类型（KW）与关键词，供发送层更新。
            return format!(
                "{IMAGE_SEND_PREFIX}|KW:{}|{}",
                matched.matched_keyword, rule.image_url
            );
        }
        rule.reply
            .replace("{send_user_name}", send_user_name)
            .replace("{send_user_id}", send_user_id)
            .replace("{send_message}", send_message)
            .replace("{item_id}", item_id)
    }
}

impl Default for KeywordMatcher {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(keyword: &str, reply: &str, item_id: &str) -> KeywordRule {
        KeywordRule {
            id: 0,
            account_id: String::new(),
            keyword: keyword.to_string(),
            reply: reply.to_string(),
            item_id: item_id.to_string(),
            rule_type: "text".to_string(),
            image_url: String::new(),
            item_title: String::new(),
        }
    }

    #[test]
    fn matches_global_keyword() {
        let matcher = KeywordMatcher::new(vec![rule("在吗", "在的", "")]);
        let found = matcher.match_message("老板在吗", None).expect("matched");
        assert_eq!(found.matched_keyword, "在吗");
    }

    #[test]
    fn item_specific_rule_wins() {
        let matcher = KeywordMatcher::new(vec![
            rule("价格", "全局价格回复", ""),
            rule("价格", "商品专属回复", "item-1"),
        ]);
        let found = matcher
            .match_message("价格多少", Some("item-1"))
            .expect("matched");
        assert_eq!(found.rule.reply, "商品专属回复");
    }

    #[test]
    fn multi_line_keyword_matches_any_line() {
        let matcher = KeywordMatcher::new(vec![rule("在吗\n你好", "回复", "")]);
        assert!(matcher.match_message("你好", None).is_some());
        assert!(matcher.match_message("在吗", None).is_some());
    }

    #[test]
    fn renders_variable_replacement() {
        let matcher = KeywordMatcher::new(vec![rule(
            "你好",
            "你好{send_user_name}，商品{item_id}",
            "",
        )]);
        let found = matcher.match_message("你好", None).expect("matched");
        let rendered = KeywordMatcher::render_reply(&found, "小明", "uid", "你好", "it-9");
        assert_eq!(rendered, "你好小明，商品it-9");
    }

    #[test]
    fn renders_image_instruction() {
        let image_rule = KeywordRule {
            id: 0,
            account_id: String::new(),
            keyword: "图".to_string(),
            reply: String::new(),
            item_id: String::new(),
            rule_type: "image".to_string(),
            image_url: "https://x/y.png".to_string(),
            item_title: String::new(),
        };
        let matcher = KeywordMatcher::new(vec![image_rule]);
        let found = matcher.match_message("发图", None).expect("matched");
        let rendered = KeywordMatcher::render_reply(&found, "", "", "", "");
        assert!(rendered.starts_with(IMAGE_SEND_PREFIX));
        assert!(rendered.contains("y.png"));
    }

    #[test]
    fn no_match_returns_none() {
        let matcher = KeywordMatcher::new(vec![rule("在吗", "在的", "")]);
        assert!(matcher.match_message("多少钱", None).is_none());
    }
}
