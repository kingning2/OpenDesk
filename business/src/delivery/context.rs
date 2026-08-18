//! 发货检查上下文 — 规则检查所需信息 + 数据访问 Port。

use super::data::DeliveryDataSource;

/// 发货检查上下文，传递给每条规则的 `check` 方法。
pub struct DeliveryCheckContext<'a> {
    /// 卖家账号 ID（xy_accounts.account_id）。
    pub account_id: &'a str,
    /// 卖家 Cookie 字符串。
    pub cookies_str: &'a str,
    /// 订单号。
    pub order_no: &'a str,
    /// 买家用户 ID。
    pub buyer_id: &'a str,
    /// 商品 ID（可能为空）。
    pub item_id: Option<&'a str>,
    /// 聊天会话 ID（可能为空）。
    pub chat_id: Option<&'a str>,
    /// 日志前缀。
    pub log_prefix: &'a str,
    /// 规则专属参数（从数据库 config 字段加载）。
    pub rule_config: &'a serde_json::Value,
    /// 卖家账号主键（用于本地订单表等）。
    pub account_pk: Option<i64>,
    /// 卖家所属用户 ID（跨账号规则使用）。
    pub owner_id: Option<i64>,
    /// 数据访问 Port（业务层注入；规则不得直接访问存储）。
    pub data: &'a dyn DeliveryDataSource,
}

impl<'a> DeliveryCheckContext<'a> {
    /// 便捷：取规则配置中某字段（缺省返回默认值）。
    pub fn config_bool(&self, key: &str, default: bool) -> bool {
        self.rule_config
            .get(key)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(default)
    }

    /// 便捷：取规则配置中某数值字段（缺省返回默认值）。
    pub fn config_u32(&self, key: &str, default: u32) -> u32 {
        self.rule_config
            .get(key)
            .and_then(serde_json::Value::as_u64)
            .map(|v| v as u32)
            .unwrap_or(default)
    }

    /// 日志前缀（未传时用账号 ID 兜底）。
    pub fn prefix(&self) -> String {
        if self.log_prefix.is_empty() {
            format!("【{}】", self.account_id)
        } else {
            self.log_prefix.to_string()
        }
    }
}
