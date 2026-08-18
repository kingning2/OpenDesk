//! 买家已有订单规则 — 同一买家在当前卖家下已有其他订单时拦截。

use crate::delivery::base::{DeliveryRule, RuleCheckResult};
use crate::delivery::context::DeliveryCheckContext;
use common::OpenDeskResult;

/// 买家已有订单规则：同一买家在同一卖家下已有其他订单时禁止发货。
pub struct BuyerHasOrderRule;

impl DeliveryRule for BuyerHasOrderRule {
    fn rule_code(&self) -> &str {
        "buyer_has_order"
    }

    fn rule_name(&self) -> &str {
        "买家已有订单"
    }

    fn rule_description(&self) -> &str {
        "检查买家在当前卖家下是否已有其他订单，有则禁止发货"
    }

    fn default_config(&self) -> serde_json::Value {
        // same_item_only: 是否仅限同商品订单才算命中。
        serde_json::json!({ "same_item_only": false })
    }

    fn check(&self, context: &DeliveryCheckContext) -> OpenDeskResult<RuleCheckResult> {
        let same_item_only = context.config_bool("same_item_only", false);
        let prefix = context.prefix();
        let item_id = if same_item_only {
            context.item_id
        } else {
            None
        };

        // 查询异常 → fail-open 放行。
        let order_count = match context.data.count_buyer_orders(
            context.account_id,
            context.buyer_id,
            context.order_no,
            item_id,
        ) {
            Ok(count) => count,
            Err(error) => {
                tracing::error!(prefix = %prefix, %error, "买家已有订单规则：查询异常，放行");
                return Ok(RuleCheckResult::pass(self.rule_code(), self.rule_name()));
            }
        };

        if order_count > 0 {
            let reason = format!("买家已有{order_count}笔其他订单，禁止发货");
            tracing::info!(
                prefix = %prefix,
                buyer_id = %context.buyer_id,
                order_count,
                same_item_only,
                "买家已有订单规则命中"
            );
            return Ok(
                RuleCheckResult::hit(self.rule_code(), self.rule_name(), reason)
                    .with_extra("order_count", order_count)
                    .with_extra("same_item_only", same_item_only),
            );
        }

        tracing::info!(
            prefix = %prefix,
            buyer_id = %context.buyer_id,
            "买家已有订单规则通过"
        );
        Ok(RuleCheckResult::pass(self.rule_code(), self.rule_name()).with_extra("order_count", 0))
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
        ) -> OpenDeskResult<u32> {
            Ok(self.0)
        }
        fn count_owner_orders(
            &self,
            _owner_id: i64,
            _buyer_id: &str,
            _exclude_order_no: &str,
            _item_id: Option<&str>,
        ) -> OpenDeskResult<u32> {
            Ok(0)
        }
        fn count_unconfirmed_orders(
            &self,
            _account_id: &str,
            _buyer_id: &str,
            _exclude_order_no: &str,
            _item_id: Option<&str>,
        ) -> OpenDeskResult<u32> {
            Ok(0)
        }
        fn find_blacklist(
            &self,
            _owner_id: i64,
            _account_id: &str,
            _buyer_id: &str,
            _item_id: Option<&str>,
        ) -> OpenDeskResult<Option<crate::delivery::data::BlacklistRecord>> {
            Ok(None)
        }
        fn fetch_buyer_rate_count(&self, _buyer_id: &str) -> OpenDeskResult<u32> {
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
            item_id: Some("item-9"),
            chat_id: None,
            log_prefix: "",
            rule_config: config,
            account_pk: None,
            owner_id: None,
            data,
        }
    }

    #[test]
    fn hits_when_buyer_has_orders() {
        let data = Data(2);
        let config = json!({});
        let result = BuyerHasOrderRule
            .check(&ctx(&config, &data))
            .expect("check");
        assert!(result.hit);
        assert_eq!(result.extra_data["order_count"], 2);
    }

    #[test]
    fn passes_when_no_orders() {
        let data = Data(0);
        let config = json!({});
        let result = BuyerHasOrderRule
            .check(&ctx(&config, &data))
            .expect("check");
        assert!(!result.hit);
    }
}
