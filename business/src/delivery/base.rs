//! 规则基类与检查结果。

use crate::delivery::context::DeliveryCheckContext;
use common::DingDaResult;

/// 规则检查结果。
#[derive(Debug, Clone)]
pub struct RuleCheckResult {
    /// 是否命中（true = 应拦截）。
    pub hit: bool,
    /// 规则唯一编码。
    pub rule_code: String,
    /// 规则中文名称。
    pub rule_name: String,
    /// 命中原因描述（写入订单 delivery_fail_reason）。
    pub reason: String,
    /// 附加数据（评价数 / 订单数等，供日志与前端展示）。
    pub extra_data: serde_json::Value,
}

impl RuleCheckResult {
    pub fn hit(rule_code: &str, rule_name: &str, reason: impl Into<String>) -> Self {
        Self {
            hit: true,
            rule_code: rule_code.to_string(),
            rule_name: rule_name.to_string(),
            reason: reason.into(),
            extra_data: serde_json::Value::Object(Default::default()),
        }
    }

    pub fn pass(rule_code: &str, rule_name: &str) -> Self {
        Self {
            hit: false,
            rule_code: rule_code.to_string(),
            rule_name: rule_name.to_string(),
            reason: String::new(),
            extra_data: serde_json::Value::Object(Default::default()),
        }
    }

    /// 追加附加数据（链式）。
    pub fn with_extra(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        if let Some(map) = self.extra_data.as_object_mut() {
            map.insert(key.to_string(), value.into());
        }
        self
    }
}

/// 禁止发货规则 trait — 所有具体规则实现此接口。
///
/// 设计约束：
/// - `check` 只做判断，不执行拦截动作（发消息 / 关单由引擎统一处理）；
/// - 数据访问通过 [`super::context::DeliveryCheckContext`] 携带的
///   [`super::data::DeliveryDataSource`] Port 完成，规则本身可单测。
pub trait DeliveryRule: Send + Sync {
    /// 规则唯一编码（存数据库用）。
    fn rule_code(&self) -> &str;

    /// 规则中文名称。
    fn rule_name(&self) -> &str;

    /// 规则描述（前端展示用）。
    fn rule_description(&self) -> &str {
        ""
    }

    /// 规则默认参数配置（前端初始化用）。
    fn default_config(&self) -> serde_json::Value {
        serde_json::Value::Object(Default::default())
    }

    /// 执行规则检查。
    fn check(&self, context: &DeliveryCheckContext) -> DingDaResult<RuleCheckResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_result_shape() {
        let result = RuleCheckResult::hit("buyer_credit_zero", "买家信用度检查", "评价数为0")
            .with_extra("total_count", 0);
        assert!(result.hit);
        assert_eq!(result.rule_code, "buyer_credit_zero");
        assert_eq!(result.extra_data["total_count"], 0);
    }

    #[test]
    fn pass_result_is_not_hit() {
        let result = RuleCheckResult::pass("r", "规则");
        assert!(!result.hit);
        assert!(result.reason.is_empty());
    }
}
