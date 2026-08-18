//! 规则注册表 — rule_code → 规则实例。

use super::base::DeliveryRule;
use super::rules::{
    BuyerCreditRule, BuyerHasOrderGlobalRule, BuyerHasOrderRule, BuyerUnconfirmedRule,
    PersonalBlacklistRule,
};

/// 注册表类型：rule_code → 规则构造器。
type RuleConstructor = fn() -> Box<dyn DeliveryRule>;

/// 所有可用规则的注册表。新增规则只需在此注册。
const RULE_REGISTRY: &[(&str, RuleConstructor)] = &[
    ("buyer_credit_zero", || Box::new(BuyerCreditRule)),
    ("buyer_has_order", || Box::new(BuyerHasOrderRule)),
    ("buyer_has_order_global", || {
        Box::new(BuyerHasOrderGlobalRule)
    }),
    ("buyer_unconfirmed", || Box::new(BuyerUnconfirmedRule)),
    ("personal_blacklist", || Box::new(PersonalBlacklistRule)),
];

/// 规则注册表访问器。
pub struct DeliveryRuleRegistry;

impl DeliveryRuleRegistry {
    /// 按 rule_code 构造规则实例；未注册的编码返回 `None`。
    pub fn instance(rule_code: &str) -> Option<Box<dyn DeliveryRule>> {
        RULE_REGISTRY
            .iter()
            .find(|(code, _)| *code == rule_code)
            .map(|(_, constructor)| constructor())
    }

    /// 全部规则元信息（前端展示用）。
    pub fn all_metadata() -> Vec<RuleMetadata> {
        RULE_REGISTRY
            .iter()
            .map(|(code, constructor)| {
                let rule = constructor();
                RuleMetadata {
                    rule_code: code.to_string(),
                    rule_name: rule.rule_name().to_string(),
                    rule_description: rule.rule_description().to_string(),
                    default_config: rule.default_config(),
                }
            })
            .collect()
    }
}

/// 规则元信息。
#[derive(Debug, Clone, serde::Serialize)]
pub struct RuleMetadata {
    pub rule_code: String,
    pub rule_name: String,
    pub rule_description: String,
    pub default_config: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_returns_known_rules() {
        for code in [
            "buyer_credit_zero",
            "buyer_has_order",
            "buyer_has_order_global",
            "buyer_unconfirmed",
            "personal_blacklist",
        ] {
            let rule = DeliveryRuleRegistry::instance(code).expect("registered rule");
            assert_eq!(rule.rule_code(), code);
        }
    }

    #[test]
    fn registry_unknown_returns_none() {
        assert!(DeliveryRuleRegistry::instance("not_a_rule").is_none());
    }

    #[test]
    fn metadata_covers_all_rules() {
        let metadata = DeliveryRuleRegistry::all_metadata();
        assert_eq!(metadata.len(), 5);
        assert!(metadata.iter().all(|m| !m.rule_code.is_empty()));
    }
}
