//! 数据访问 Port — 规则引擎与存储/外部 API 的解耦层。
//!
//! 业务层（Tauri 壳 / 未来 Web 服务）实现这些接口注入，
//! 规则本身保持纯逻辑、可单测。查询异常返回 `Err`（规则层 fail-open 放行）。

use common::OpenDeskResult;
/// 黑名单命中记录。
#[derive(Debug, Clone)]
pub struct BlacklistRecord {
    pub id: i64,
    pub account_id: Option<String>,
    pub item_id: Option<String>,
    pub reason: Option<String>,
}

impl BlacklistRecord {
    /// 命中级别：商品级 > 账户级 > 用户级。
    pub fn level(&self) -> &'static str {
        match (&self.account_id, &self.item_id) {
            (Some(_), Some(_)) => "商品级",
            (Some(_), None) => "账户级",
            _ => "用户级",
        }
    }
}

/// 发货规则数据源 Port。
pub trait DeliveryDataSource: Send + Sync {
    /// 统计买家在指定卖家账号下的订单数（排除当前订单号）。
    fn count_buyer_orders(
        &self,
        account_id: &str,
        buyer_id: &str,
        exclude_order_no: &str,
        item_id: Option<&str>,
    ) -> OpenDeskResult<u32>;

    /// 统计买家在指定用户名下所有账号的订单数（排除当前订单号）。
    fn count_owner_orders(
        &self,
        owner_id: i64,
        buyer_id: &str,
        exclude_order_no: &str,
        item_id: Option<&str>,
    ) -> OpenDeskResult<u32>;

    /// 统计买家未确认收货订单数（已发货未收货）。
    fn count_unconfirmed_orders(
        &self,
        account_id: &str,
        buyer_id: &str,
        exclude_order_no: &str,
        item_id: Option<&str>,
    ) -> OpenDeskResult<u32>;

    /// 查询买家个人黑名单（三级匹配：商品级 > 账户级 > 用户级）。
    fn find_blacklist(
        &self,
        owner_id: i64,
        account_id: &str,
        buyer_id: &str,
        item_id: Option<&str>,
    ) -> OpenDeskResult<Option<BlacklistRecord>>;

    /// 查询买家被评价总数（`mtop.idle.web.trade.rate.list`）。
    /// 接口异常返回 `Err`（规则 fail-open）。
    fn fetch_buyer_rate_count(&self, buyer_id: &str) -> OpenDeskResult<u32>;
}
