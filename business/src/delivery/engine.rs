//! 规则执行引擎 — 加载启用规则 → 按优先级 → 首条命中即停。

use crate::delivery::base::DeliveryRule;
use crate::delivery::context::DeliveryCheckContext;
use crate::delivery::data::DeliveryDataSource;
use crate::delivery::registry::DeliveryRuleRegistry;

/// 规则配置（从数据库 `xy_delivery_block_rules` 加载，业务层提供）。
#[derive(Debug, Clone)]
pub struct RuleConfig {
    pub rule_code: String,
    /// 是否启用。
    pub enabled: bool,
    /// 执行优先级（越小越先执行）。
    pub priority: i32,
    /// 配置的禁止发货原因（发给买家）。
    pub block_reason: String,
    /// 命中后是否主动关闭订单。
    pub auto_close_order: bool,
    /// 关闭订单后是否只发卡券。
    pub only_card_after_close: bool,
    /// 排除商品 ID 列表（命中则跳过本规则）。
    pub excluded_item_ids: Vec<String>,
    /// 规则专属参数。
    pub config: serde_json::Value,
}

/// 引擎执行结果。
#[derive(Debug, Clone)]
pub struct EngineResult {
    /// 是否有规则命中。
    pub hit: bool,
    /// 命中的规则编码。
    pub rule_code: Option<String>,
    /// 命中的规则名称。
    pub rule_name: Option<String>,
    /// 命中原因（写入订单 delivery_fail_reason）。
    pub reason: String,
    /// 配置的禁止发货原因（发给买家）。
    pub block_reason: String,
    /// 是否主动关闭订单。
    pub auto_close_order: bool,
    /// 关闭后是否只发卡券。
    pub only_card_after_close: bool,
    /// 规则附加数据。
    pub extra_data: serde_json::Value,
}

impl EngineResult {
    fn allow() -> Self {
        Self {
            hit: false,
            rule_code: None,
            rule_name: None,
            reason: String::new(),
            block_reason: String::new(),
            auto_close_order: false,
            only_card_after_close: false,
            extra_data: serde_json::Value::Object(Default::default()),
        }
    }

    fn block(
        rule: &dyn DeliveryRule,
        config: &RuleConfig,
        reason: String,
        extra_data: serde_json::Value,
    ) -> Self {
        Self {
            hit: true,
            rule_code: Some(rule.rule_code().to_string()),
            rule_name: Some(rule.rule_name().to_string()),
            reason,
            block_reason: config.block_reason.clone(),
            auto_close_order: config.auto_close_order,
            only_card_after_close: config.only_card_after_close,
            extra_data,
        }
    }
}

/// 引擎执行入参（统一上下文，替代长参数列表）。
pub struct DeliveryRequest<'a> {
    pub account_id: &'a str,
    pub cookies_str: &'a str,
    pub order_no: &'a str,
    pub buyer_id: &'a str,
    pub item_id: Option<&'a str>,
    pub chat_id: Option<&'a str>,
    pub log_prefix: &'a str,
    pub account_pk: Option<i64>,
    pub owner_id: Option<i64>,
    pub rule_configs: &'a [RuleConfig],
}

/// 规则执行引擎。
///
/// 流程（对齐 Python 版 `execute_rules`）：
/// 1. 加载启用规则（按 priority 升序）；
/// 2. 无启用规则 → 直接放行；
/// 3. 逐条执行：排除商品命中 → 跳过；规则命中 → 首条即停；
/// 4. 全部通过 → 放行。
pub struct DeliveryEngine<'a> {
    data: &'a dyn DeliveryDataSource,
}

impl<'a> DeliveryEngine<'a> {
    pub fn new(data: &'a dyn DeliveryDataSource) -> Self {
        Self { data }
    }

    /// 执行全部启用规则。
    pub fn execute(&self, request: &DeliveryRequest<'_>) -> EngineResult {
        let DeliveryRequest {
            account_id,
            cookies_str,
            order_no,
            buyer_id,
            item_id,
            chat_id,
            log_prefix,
            account_pk,
            owner_id,
            rule_configs,
        } = *request;
        // 1. 仅启用规则，按 priority 升序。
        let mut enabled: Vec<&RuleConfig> = rule_configs
            .iter()
            .filter(|config| config.enabled)
            .collect();
        enabled.sort_by_key(|config| config.priority);

        // 2. 无启用规则 → 放行。
        if enabled.is_empty() {
            tracing::info!(prefix = %log_prefix, order_no, "无已启用的禁止发货规则，放行");
            return EngineResult::allow();
        }

        // 3. 逐条执行，首条命中即停。
        for config in enabled {
            // 3.1 排除商品列表检查。
            if let Some(item_id) = item_id {
                if config
                    .excluded_item_ids
                    .iter()
                    .any(|excluded| excluded == item_id)
                {
                    tracing::info!(
                        prefix = %log_prefix,
                        item_id,
                        rule_code = %config.rule_code,
                        "商品命中规则排除列表，跳过本规则"
                    );
                    continue;
                }
            }

            // 3.2 获取规则实例。
            let Some(rule) = DeliveryRuleRegistry::instance(&config.rule_code) else {
                tracing::warn!(prefix = %log_prefix, rule_code = %config.rule_code, "未注册的规则编码，跳过");
                continue;
            };

            // 3.3 构建上下文并检查。
            let context = DeliveryCheckContext {
                account_id,
                cookies_str,
                order_no,
                buyer_id,
                item_id,
                chat_id,
                log_prefix,
                rule_config: &config.config,
                account_pk,
                owner_id,
                data: self.data,
            };
            let result = match rule.check(&context) {
                Ok(result) => result,
                Err(error) => {
                    tracing::error!(
                        prefix = %log_prefix,
                        rule_code = %config.rule_code,
                        %error,
                        "规则执行异常，跳过本规则"
                    );
                    continue;
                }
            };

            // 3.4 命中 → 返回该规则的配置。
            if result.hit {
                tracing::warn!(
                    prefix = %log_prefix,
                    rule_code = %config.rule_code,
                    rule_name = %result.rule_name,
                    order_no,
                    buyer_id,
                    reason = %result.reason,
                    "❌ 命中禁止发货规则"
                );
                return EngineResult::block(
                    rule.as_ref(),
                    config,
                    result.reason,
                    result.extra_data,
                );
            }
        }

        // 4. 全部通过 → 放行。
        tracing::info!(prefix = %log_prefix, order_no, buyer_id, "所有规则检查通过，放行");
        EngineResult::allow()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delivery::data::BlacklistRecord;
    use common::DingDaResult;

    struct Data {
        rate: u32,
        orders: u32,
        blacklist: Option<BlacklistRecord>,
    }

    impl DeliveryDataSource for Data {
        fn count_buyer_orders(
            &self,
            _account_id: &str,
            _buyer_id: &str,
            _exclude_order_no: &str,
            _item_id: Option<&str>,
        ) -> DingDaResult<u32> {
            Ok(self.orders)
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
            Ok(self.blacklist.clone())
        }
        fn fetch_buyer_rate_count(&self, _buyer_id: &str) -> DingDaResult<u32> {
            Ok(self.rate)
        }
    }

    fn rule(code: &str, priority: i32) -> RuleConfig {
        RuleConfig {
            rule_code: code.to_string(),
            enabled: true,
            priority,
            block_reason: "平台规则限制".to_string(),
            auto_close_order: false,
            only_card_after_close: false,
            excluded_item_ids: Vec::new(),
            config: serde_json::json!({}),
        }
    }

    fn engine(data: &dyn DeliveryDataSource) -> DeliveryEngine<'_> {
        DeliveryEngine::new(data)
    }

    /// 构造执行入参（默认 buyer-1 / 无 item / owner 7）。
    fn req<'a>(rules: &'a [RuleConfig], item_id: Option<&'a str>) -> DeliveryRequest<'a> {
        DeliveryRequest {
            account_id: "acc-1",
            cookies_str: "",
            order_no: "o-1",
            buyer_id: "buyer-1",
            item_id,
            chat_id: None,
            log_prefix: "",
            account_pk: None,
            owner_id: Some(7),
            rule_configs: rules,
        }
    }

    #[test]
    fn allows_when_no_enabled_rules() {
        let data = Data {
            rate: 0,
            orders: 9,
            blacklist: None,
        };
        let result = engine(&data).execute(&req(&[], None));
        assert!(!result.hit);
    }

    #[test]
    fn blocks_on_first_hit_by_priority() {
        // buyer_credit_zero（priority 1）先执行且命中 → 首条即停，buyer_has_order 不再执行。
        let data = Data {
            rate: 0,
            orders: 9,
            blacklist: None,
        };
        let rules = vec![rule("buyer_has_order", 2), rule("buyer_credit_zero", 1)];
        let result = engine(&data).execute(&req(&rules, None));
        assert!(result.hit);
        assert_eq!(result.rule_code.as_deref(), Some("buyer_credit_zero"));
        assert_eq!(result.block_reason, "平台规则限制");
    }

    #[test]
    fn blocked_rule_carries_extra_data() {
        let data = Data {
            rate: 0,
            orders: 0,
            blacklist: None,
        };
        let rules = vec![rule("buyer_credit_zero", 1)];
        let result = engine(&data).execute(&req(&rules, None));
        assert!(result.hit);
        assert_eq!(result.extra_data["total_count"], 0);
    }

    #[test]
    fn excluded_item_skips_rule() {
        let data = Data {
            rate: 0,
            orders: 0,
            blacklist: None,
        };
        let mut credit = rule("buyer_credit_zero", 1);
        credit.excluded_item_ids = vec!["item-9".to_string()];
        let result = engine(&data).execute(&req(&[credit], Some("item-9")));
        assert!(!result.hit);
    }

    #[test]
    fn disabled_rules_ignored() {
        let data = Data {
            rate: 0,
            orders: 0,
            blacklist: None,
        };
        let mut disabled = rule("buyer_credit_zero", 1);
        disabled.enabled = false;
        let result = engine(&data).execute(&req(&[disabled], None));
        assert!(!result.hit);
    }

    #[test]
    fn unknown_rule_code_skipped() {
        let data = Data {
            rate: 0,
            orders: 0,
            blacklist: None,
        };
        let rules = vec![rule("not_registered", 1)];
        let result = engine(&data).execute(&req(&rules, None));
        assert!(!result.hit);
    }

    #[test]
    fn personal_blacklist_hits() {
        let data = Data {
            rate: 5,
            orders: 0,
            blacklist: Some(BlacklistRecord {
                id: 1,
                account_id: Some("acc-1".to_string()),
                item_id: None,
                reason: Some("差评买家".to_string()),
            }),
        };
        let rules = vec![rule("personal_blacklist", 1)];
        let result = engine(&data).execute(&req(&rules, None));
        assert!(result.hit);
        assert_eq!(result.rule_code.as_deref(), Some("personal_blacklist"));
    }
}
