//! 个人黑名单规则 — 三级匹配（商品级 > 账户级 > 用户级）。

use crate::delivery::base::{DeliveryRule, RuleCheckResult};
use crate::delivery::context::DeliveryCheckContext;
use common::DingDaResult;

/// 个人黑名单规则：买家在个人黑名单中时禁止发货。
pub struct PersonalBlacklistRule;

impl DeliveryRule for PersonalBlacklistRule {
    fn rule_code(&self) -> &str {
        "personal_blacklist"
    }

    fn rule_name(&self) -> &str {
        "个人黑名单"
    }

    fn rule_description(&self) -> &str {
        "检查买家是否在个人黑名单中（支持商品级、账户级、用户级匹配）"
    }

    fn check(&self, context: &DeliveryCheckContext) -> DingDaResult<RuleCheckResult> {
        let prefix = context.prefix();

        // owner_id 缺失：无法查询用户级黑名单，fail-open 放行。
        let Some(owner_id) = context.owner_id else {
            tracing::warn!(prefix = %prefix, "个人黑名单规则：owner_id 缺失，放行");
            return Ok(RuleCheckResult::pass(self.rule_code(), self.rule_name()));
        };

        let record = match context.data.find_blacklist(
            owner_id,
            context.account_id,
            context.buyer_id,
            context.item_id,
        ) {
            Ok(record) => record,
            Err(error) => {
                tracing::error!(prefix = %prefix, %error, "个人黑名单规则：查询异常，放行");
                return Ok(RuleCheckResult::pass(self.rule_code(), self.rule_name()));
            }
        };

        let Some(record) = record else {
            tracing::info!(
                prefix = %prefix,
                buyer_id = %context.buyer_id,
                "个人黑名单规则通过"
            );
            return Ok(RuleCheckResult::pass(self.rule_code(), self.rule_name()));
        };

        let level = record.level();
        let reason = match record.reason {
            Some(text) if !text.is_empty() => {
                format!("买家在个人黑名单中（{level}），原因：{text}")
            }
            _ => format!("买家在个人黑名单中（{level}）"),
        };
        tracing::info!(
            prefix = %prefix,
            buyer_id = %context.buyer_id,
            level,
            blacklist_id = record.id,
            "个人黑名单规则命中"
        );
        Ok(
            RuleCheckResult::hit(self.rule_code(), self.rule_name(), reason)
                .with_extra("blacklist_id", record.id)
                .with_extra("level", level),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delivery::context::DeliveryCheckContext;
    use crate::delivery::data::{BlacklistRecord, DeliveryDataSource};

    struct Data(Option<BlacklistRecord>);

    impl DeliveryDataSource for Data {
        fn count_buyer_orders(
            &self,
            _account_id: &str,
            _buyer_id: &str,
            _exclude_order_no: &str,
            _item_id: Option<&str>,
        ) -> DingDaResult<u32> {
            Ok(0)
        }
        fn count_owner_orders(
            &self,
            _owner_id: i64,
            _buyer_id: &str,
            _exclude_order_no: &str,
            _item_id: Option<&str>,
        ) -> DingDaResult<u32> {
            Ok(0)
        }
        fn count_unconfirmed_orders(
            &self,
            _account_id: &str,
            _buyer_id: &str,
            _exclude_order_no: &str,
            _item_id: Option<&str>,
        ) -> DingDaResult<u32> {
            Ok(0)
        }
        fn find_blacklist(
            &self,
            _owner_id: i64,
            _account_id: &str,
            _buyer_id: &str,
            _item_id: Option<&str>,
        ) -> DingDaResult<Option<BlacklistRecord>> {
            Ok(self.0.clone())
        }
        fn fetch_buyer_rate_count(&self, _buyer_id: &str) -> DingDaResult<u32> {
            Ok(10)
        }
    }

    fn ctx<'a>(
        owner_id: Option<i64>,
        data: &'a dyn DeliveryDataSource,
    ) -> DeliveryCheckContext<'a> {
        static EMPTY_CONFIG: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
        DeliveryCheckContext {
            account_id: "acc-1",
            cookies_str: "",
            order_no: "o-1",
            buyer_id: "buyer-1",
            item_id: Some("item-9"),
            chat_id: None,
            log_prefix: "",
            rule_config: EMPTY_CONFIG.get_or_init(|| serde_json::json!({})),
            account_pk: None,
            owner_id,
            data,
        }
    }

    fn record(account: Option<&str>, item: Option<&str>) -> BlacklistRecord {
        BlacklistRecord {
            id: 1,
            account_id: account.map(|s| s.to_string()),
            item_id: item.map(|s| s.to_string()),
            reason: Some("恶意买家".to_string()),
        }
    }

    #[test]
    fn hits_when_blacklisted() {
        let data = Data(Some(record(Some("acc-1"), Some("item-9"))));
        let result = PersonalBlacklistRule
            .check(&ctx(Some(7), &data))
            .expect("check");
        assert!(result.hit);
        assert_eq!(result.extra_data["level"], "商品级");
        assert!(result.reason.contains("恶意买家"));
    }

    #[test]
    fn passes_when_not_blacklisted() {
        let data = Data(None);
        let result = PersonalBlacklistRule
            .check(&ctx(Some(7), &data))
            .expect("check");
        assert!(!result.hit);
    }

    #[test]
    fn passes_when_owner_missing() {
        let data = Data(Some(record(Some("acc-1"), Some("item-9"))));
        let result = PersonalBlacklistRule
            .check(&ctx(None, &data))
            .expect("check");
        assert!(!result.hit);
    }

    #[test]
    fn level_classification() {
        assert_eq!(record(Some("a"), Some("i")).level(), "商品级");
        assert_eq!(record(Some("a"), None).level(), "账户级");
        assert_eq!(record(None, None).level(), "用户级");
    }
}
