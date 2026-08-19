//! 买家信用度规则 — 被评价总数 <= 阈值（默认 0）时拦截。

use crate::delivery::base::{DeliveryRule, RuleCheckResult};
use crate::delivery::context::DeliveryCheckContext;
use common::DingDaResult;

/// 买家信用度检查规则：评价数为 0（或低于阈值）时禁止发货。
pub struct BuyerCreditRule;

impl DeliveryRule for BuyerCreditRule {
    fn rule_code(&self) -> &str {
        "buyer_credit_zero"
    }

    fn rule_name(&self) -> &str {
        "买家信用度检查"
    }

    fn rule_description(&self) -> &str {
        "检查买家被评价总数，评价数为0（或低于设定阈值）时禁止发货"
    }

    fn default_config(&self) -> serde_json::Value {
        serde_json::json!({ "threshold": 0 })
    }

    fn check(&self, context: &DeliveryCheckContext) -> DingDaResult<RuleCheckResult> {
        let threshold = context.config_u32("threshold", 0);
        let prefix = context.prefix();

        // 数据源异常 → fail-open 放行。
        let total_count = match context.data.fetch_buyer_rate_count(context.buyer_id) {
            Ok(count) => count,
            Err(error) => {
                warn!(
                    prefix = %prefix,
                    buyer_id = %context.buyer_id,
                    %error,
                    "买家信用度规则：评价接口异常，无法确认，跳过本规则"
                );
                return Ok(RuleCheckResult::pass(self.rule_code(), self.rule_name())
                    .with_extra("total_count", -1));
            }
        };

        if total_count <= threshold {
            let reason = format!("买家评价数为{total_count}（阈值{threshold}），已禁止发货");
            info!(
                prefix = %prefix,
                buyer_id = %context.buyer_id,
                total_count,
                threshold,
                "买家信用度规则命中"
            );
            return Ok(
                RuleCheckResult::hit(self.rule_code(), self.rule_name(), reason)
                    .with_extra("total_count", total_count)
                    .with_extra("threshold", threshold),
            );
        }

        info!(
            prefix = %prefix,
            buyer_id = %context.buyer_id,
            total_count,
            threshold,
            "买家信用度规则通过"
        );
        Ok(RuleCheckResult::pass(self.rule_code(), self.rule_name())
            .with_extra("total_count", total_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delivery::context::DeliveryCheckContext;
    use crate::delivery::data::DeliveryDataSource;
    use serde_json::json;

    struct Data(u32, bool);

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
        ) -> DingDaResult<Option<crate::delivery::data::BlacklistRecord>> {
            Ok(None)
        }
        fn fetch_buyer_rate_count(&self, _buyer_id: &str) -> DingDaResult<u32> {
            if self.1 {
                Err("api down".to_string().into())
            } else {
                Ok(self.0)
            }
        }
    }

    fn ctx<'a>(
        buyer_id: &'a str,
        config: &'a serde_json::Value,
        data: &'a dyn DeliveryDataSource,
    ) -> DeliveryCheckContext<'a> {
        DeliveryCheckContext {
            account_id: "acc-1",
            cookies_str: "",
            order_no: "o-1",
            buyer_id,
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
    fn hits_when_zero_ratings() {
        let data = Data(0, false);
        let config = json!({});
        let result = BuyerCreditRule
            .check(&ctx("buyer-1", &config, &data))
            .expect("check");
        assert!(result.hit);
        assert_eq!(result.extra_data["total_count"], 0);
    }

    #[test]
    fn passes_when_above_threshold() {
        let data = Data(5, false);
        let config = json!({});
        let result = BuyerCreditRule
            .check(&ctx("buyer-1", &config, &data))
            .expect("check");
        assert!(!result.hit);
    }

    #[test]
    fn passes_with_custom_threshold() {
        let data = Data(2, false);
        let config = json!({ "threshold": 3 });
        let result = BuyerCreditRule
            .check(&ctx("buyer-1", &config, &data))
            .expect("check");
        assert!(result.hit);
    }

    #[test]
    fn fails_open_on_api_error() {
        let data = Data(0, true);
        let config = json!({});
        let result = BuyerCreditRule
            .check(&ctx("buyer-1", &config, &data))
            .expect("check");
        assert!(!result.hit);
    }
}
