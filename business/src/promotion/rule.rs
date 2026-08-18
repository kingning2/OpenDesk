//! 返佣规则领域模型。

/// 规则启用状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleStatus {
    Enabled,
    Disabled,
}

impl RuleStatus {
    pub fn as_bool(&self) -> bool {
        matches!(self, RuleStatus::Enabled)
    }

    pub fn from_bool(enabled: bool) -> Self {
        if enabled {
            RuleStatus::Enabled
        } else {
            RuleStatus::Disabled
        }
    }
}

/// 选品规则（对齐 `fy_product_rules`）。
#[derive(Debug, Clone)]
pub struct ProductRule {
    pub id: i64,
    pub owner_id: i64,
    pub account_id: String,
    pub rule_name: String,
    /// 商品类目（可为空）。
    pub cat: Option<String>,
    /// 类目名称（展示用）。
    pub cat_name: Option<String>,
    /// 关键词（可为空）。
    pub keyword: Option<String>,
    /// 排序方式：default / 其他。
    pub sort: String,
    /// 每日选品数量。
    pub daily_count: u32,
    pub status: RuleStatus,
    pub remark: Option<String>,
}

impl ProductRule {
    /// 类目与关键词至少填一项（Python 版 create_rule 校验）。
    pub fn has_source(&self) -> bool {
        self.cat.as_deref().is_some_and(|c| !c.trim().is_empty())
            || self
                .keyword
                .as_deref()
                .is_some_and(|k| !k.trim().is_empty())
    }
}

/// 发布规则（对齐 `fy_publish_rules`）。
#[derive(Debug, Clone)]
pub struct PublishRule {
    pub id: i64,
    pub owner_id: i64,
    pub rule_name: String,
    pub account_id: String,
    /// 每日发布数量。
    pub daily_count: u32,
    pub status: RuleStatus,
    pub remark: Option<String>,
}

/// 发布规则创建入参。
#[derive(Debug, Clone)]
pub struct PublishRuleInput {
    pub owner_id: i64,
    pub rule_name: String,
    pub account_id: String,
    pub daily_count: u32,
    pub enabled: bool,
    pub remark: Option<String>,
}

/// 选品规则创建入参。
#[derive(Debug, Clone)]
pub struct ProductRuleInput {
    pub owner_id: i64,
    pub account_id: String,
    pub rule_name: String,
    pub cat: Option<String>,
    pub cat_name: Option<String>,
    pub keyword: Option<String>,
    pub sort: String,
    pub daily_count: u32,
    pub enabled: bool,
    pub remark: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn product_rule(cat: Option<&str>, keyword: Option<&str>) -> ProductRule {
        ProductRule {
            id: 1,
            owner_id: 1,
            account_id: "acc-1".to_string(),
            rule_name: "规则".to_string(),
            cat: cat.map(|s| s.to_string()),
            cat_name: None,
            keyword: keyword.map(|s| s.to_string()),
            sort: "default".to_string(),
            daily_count: 10,
            status: RuleStatus::Enabled,
            remark: None,
        }
    }

    #[test]
    fn source_requires_cat_or_keyword() {
        assert!(product_rule(Some("手机"), None).has_source());
        assert!(product_rule(None, Some("耳机")).has_source());
        assert!(!product_rule(None, None).has_source());
        assert!(!product_rule(Some("  "), Some("")).has_source());
    }

    #[test]
    fn status_roundtrip() {
        assert!(RuleStatus::Enabled.as_bool());
        assert!(!RuleStatus::Disabled.as_bool());
        assert_eq!(RuleStatus::from_bool(true), RuleStatus::Enabled);
        assert_eq!(RuleStatus::from_bool(false), RuleStatus::Disabled);
    }
}
