//! 消息过滤 — 关键词命中时跳过自动回复 / 跳过通知。
//!
//! 对齐 Python 版：过滤关键词分两类，命中即跳过对应动作。

use serde::{Deserialize, Serialize};

/// 过滤类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterType {
    /// 命中则跳过自动回复。
    SkipReply,
    /// 命中则跳过消息通知。
    SkipNotify,
}

impl FilterType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FilterType::SkipReply => "skip_reply",
            FilterType::SkipNotify => "skip_notify",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "skip_notify" => FilterType::SkipNotify,
            _ => FilterType::SkipReply,
        }
    }
}

/// 过滤规则（单条）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub owner_id: i64,
    pub filter_type: FilterType,
    pub keyword: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// 关键词过滤器 — 规则集判定。
pub struct KeywordFilter {
    rules: Vec<FilterRule>,
}

impl KeywordFilter {
    pub fn new(rules: Vec<FilterRule>) -> Self {
        Self { rules }
    }

    /// 命中任一规则返回 `true`。
    pub fn matches(&self, message: &str, filter_type: FilterType) -> bool {
        self.rules
            .iter()
            .filter(|rule| rule.filter_type == filter_type)
            .any(|rule| message.contains(&rule.keyword))
    }

    pub fn skip_reply(&self, message: &str) -> bool {
        self.matches(message, FilterType::SkipReply)
    }

    pub fn skip_notify(&self, message: &str) -> bool {
        self.matches(message, FilterType::SkipNotify)
    }
}

impl Default for KeywordFilter {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter() -> KeywordFilter {
        KeywordFilter::new(vec![
            FilterRule {
                id: 1,
                account_id: String::new(),
                owner_id: 0,
                filter_type: FilterType::SkipReply,
                keyword: "勿扰".to_string(),
                enabled: true,
            },
            FilterRule {
                id: 2,
                account_id: String::new(),
                owner_id: 0,
                filter_type: FilterType::SkipNotify,
                keyword: "广告".to_string(),
                enabled: true,
            },
        ])
    }

    #[test]
    fn skips_reply_by_keyword() {
        assert!(filter().skip_reply("我现在勿扰"));
        assert!(!filter().skip_reply("你好"));
    }

    #[test]
    fn skips_notify_by_keyword() {
        assert!(filter().skip_notify("发广告"));
        // "广告" 子串在 "广告？不需要" 中仍命中（contains 语义）。
        assert!(filter().skip_notify("广告？不需要"));
        assert!(!filter().skip_notify("你好呀"));
    }

    #[test]
    fn reply_and_notify_are_independent() {
        assert!(filter().skip_reply("勿扰勿扰"));
        assert!(!filter().skip_notify("勿扰勿扰"));
    }
}
