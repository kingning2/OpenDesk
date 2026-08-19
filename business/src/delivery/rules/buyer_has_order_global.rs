//! 买家在同用户其他账号已有订单规则 — 防跨店薅羊毛。

use crate::delivery::base::{DeliveryRule, RuleCheckResult};
use crate::delivery::context::DeliveryCheckContext;
use common::DingDaResult;

/// 买家在同用户名下所有账号已有其他订单时禁止发货。
pub struct BuyerHasOrderGlobalRule;

impl DeliveryRule for BuyerHasOrderGlobalRule {
    fn rule_code(&self) -> &str {
        "buyer_has_order_global"
    }

    fn rule_name(&self) -> &str {
        "买家在同用户其他账号已有订单"
    }

    fn rule_description(&self) -> &str {
        "检查买家在当前用户名下所有账号中是否已有其他订单，有则禁止发货"
    }

    fn default_config(&self) -> serde_json::Value {
        serde_json::json!({ "same_item_only": false })
    }

    fn check(&self, context: &DeliveryCheckContext) -> DingDaResult<RuleCheckResult> {
        let prefix = context.prefix();
        let same_item_only = context.config_bool("same_item_only", false);

        // owner_id / buyer_id 缺失保护：跨账号查询，空值会误拦大量订单，必须放行。
        let (Some(owner_id), false) = (context.owner_id, context.buyer_id.is_empty()) else {
            warn!(prefix = %prefix, "同用户已有订单规则：owner_id 或 buyer_id 缺失，放行");
            return Ok(RuleCheckResult::pass(self.rule_code(), self.rule_name()));
        };
        let item_id = if same_item_only {
            context.item_id
        } else {
            None
        };

        let order_count = match context.data.count_owner_orders(
            owner_id,
            context.buyer_id,
            context.order_no,
            item_id,
        ) {
            Ok(count) => count,
            Err(error) => {
                error!(prefix = %prefix, %error, "同用户已有订单规则：查询异常，放行");
                return Ok(RuleCheckResult::pass(self.rule_code(), self.rule_name()));
            }
        };

        if order_count > 0 {
            let reason = format!("买家在您名下其他账号已有{order_count}笔订单，禁止发货");
            info!(
                prefix = %prefix,
                buyer_id = %context.buyer_id,
                owner_id,
                order_count,
                same_item_only,
                "同用户已有订单规则命中"
            );
            return Ok(
                RuleCheckResult::hit(self.rule_code(), self.rule_name(), reason)
                    .with_extra("order_count", order_count)
                    .with_extra("same_item_only", same_item_only),
            );
        }

        info!(
            prefix = %prefix,
            buyer_id = %context.buyer_id,
            owner_id,
            "同用户已有订单规则通过"
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
            Ok(self.0)
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
        ) -> DingDaResult<Option<crate::delivery::data::BlacklistRecord>> {
            Ok(None)
        }
        fn fetch_buyer_rate_count(&self, _buyer_id: &str) -> DingDaResult<u32> {
            Ok(10)
        }
    }

    fn ctx<'a>(
        owner_id: Option<i64>,
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
            owner_id,
            data,
        }
    }

    #[test]
    fn hits_when_owner_has_orders() {
        let data = Data(3);
        let config = json!({});
        let result = BuyerHasOrderGlobalRule
            .check(&ctx(Some(7), &config, &data))
            .expect("check");
        assert!(result.hit);
    }

    #[test]
    fn passes_without_orders() {
        let data = Data(0);
        let config = json!({});
        let result = BuyerHasOrderGlobalRule
            .check(&ctx(Some(7), &config, &data))
            .expect("check");
        assert!(!result.hit);
    }

    #[test]
    fn passes_when_owner_missing() {
        let data = Data(3);
        let config = json!({});
        let result = BuyerHasOrderGlobalRule
            .check(&ctx(None, &config, &data))
            .expect("check");
        assert!(!result.hit);
    }
}
