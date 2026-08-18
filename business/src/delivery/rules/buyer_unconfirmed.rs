//! 买家存在未确认收货订单规则 — 防买家拖单/薅羊毛。

use crate::delivery::base::{DeliveryRule, RuleCheckResult};
use crate::delivery::context::DeliveryCheckContext;
use common::DingDaResult;

/// 买家存在未确认收货订单规则：未确认收货订单数 >= 阈值时禁止发货。
pub struct BuyerUnconfirmedRule;

impl DeliveryRule for BuyerUnconfirmedRule {
    fn rule_code(&self) -> &str {
        "buyer_unconfirmed"
    }

    fn rule_name(&self) -> &str {
        "买家存在未确认收货订单"
    }

    fn rule_description(&self) -> &str {
        "检查买家在当前卖家下是否有未确认收货的订单，有则禁止发货"
    }

    fn default_config(&self) -> serde_json::Value {
        // min_count: 未确认收货订单数达到多少时触发拦截。
        serde_json::json!({ "min_count": 1, "same_item_only": false })
    }

    fn check(&self, context: &DeliveryCheckContext) -> DingDaResult<RuleCheckResult> {
        let min_count = context.config_u32("min_count", 1);
        let same_item_only = context.config_bool("same_item_only", false);
        let prefix = context.prefix();
        let item_id = if same_item_only {
            context.item_id
        } else {
            None
        };

        let unconfirmed_count = match context.data.count_unconfirmed_orders(
            context.account_id,
            context.buyer_id,
            context.order_no,
            item_id,
        ) {
            Ok(count) => count,
            Err(error) => {
                tracing::error!(prefix = %prefix, %error, "未确认收货规则：查询异常，放行");
                return Ok(RuleCheckResult::pass(self.rule_code(), self.rule_name()));
            }
        };

        if unconfirmed_count >= min_count {
            let reason = format!("买家有{unconfirmed_count}笔未确认收货订单，禁止发货");
            tracing::info!(
                prefix = %prefix,
                buyer_id = %context.buyer_id,
                unconfirmed_count,
                min_count,
                "未确认收货规则命中"
            );
            return Ok(
                RuleCheckResult::hit(self.rule_code(), self.rule_name(), reason)
                    .with_extra("unconfirmed_count", unconfirmed_count)
                    .with_extra("min_count", min_count),
            );
        }

        tracing::info!(
            prefix = %prefix,
            buyer_id = %context.buyer_id,
            unconfirmed_count,
            "未确认收货规则通过"
        );
        Ok(RuleCheckResult::pass(self.rule_code(), self.rule_name())
            .with_extra("unconfirmed_count", unconfirmed_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delivery::context::DeliveryCheckContext;
    use crate::delivery::data::DeliveryDataSource;
    use serde_json::json;

    struct Data(u32);

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
            Ok(self.0)
        }
        fn find_blacklist(
            &self,
            _owner_id: i64,
            _account_id: &str,
            _buyer_id: &str,
            _item_id: Option<&str>,
        ) -> DingDaResult<Option<crate::delivery::data::BlacklistRecord>> {
            Ok(None)
        }
        fn fetch_buyer_rate_count(&self, _buyer_id: &str) -> DingDaResult<u32> {
            Ok(10)
        }
    }

    fn ctx<'a>(
        config: &'a serde_json::Value,
        data: &'a dyn DeliveryDataSource,
    ) -> DeliveryCheckContext<'a> {
        DeliveryCheckContext {
            account_id: "acc-1",
            cookies_str: "",
            order_no: "o-1",
            buyer_id: "buyer-1",
            item_id: None,
            chat_id: None,
            log_prefix: "",
            rule_config: config,
            account_pk: None,
            owner_id: None,
            data,
        }
    }

    #[test]
    fn hits_at_min_count() {
        let data = Data(1);
        let config = json!({});
        let result = BuyerUnconfirmedRule
            .check(&ctx(&config, &data))
            .expect("check");
        assert!(result.hit);
        assert_eq!(result.extra_data["unconfirmed_count"], 1);
    }

    #[test]
    fn passes_below_min_count() {
        let data = Data(0);
        let config = json!({});
        let result = BuyerUnconfirmedRule
            .check(&ctx(&config, &data))
            .expect("check");
        assert!(!result.hit);
    }

    #[test]
    fn respects_custom_min_count() {
        let data = Data(2);
        let config = json!({ "min_count": 3 });
        let result = BuyerUnconfirmedRule
            .check(&ctx(&config, &data))
            .expect("check");
        assert!(!result.hit);
    }
}
