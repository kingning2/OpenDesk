//! 发货网关 Port — 确认发货 / 消息发送 / 订单状态 / 卡券数据。

use async_trait::async_trait;

use super::card::Card;
use common::DingDaResult;

/// 确认发货结果。
#[derive(Debug, Clone)]
pub struct ConfirmResult {
    pub success: bool,
    /// 已发货过（幂等，不再发卡券）。
    pub already_delivered: bool,
    /// 账号只发卡券模式（跳过确认）。
    pub skipped_only_send_card: bool,
    pub message: String,
}

/// 发货网关 — 平台/存储操作抽象。
#[async_trait]
pub trait DeliveryGateway: Send + Sync {
    /// 确认发货（mtop `mtop.taobao.idle.logistic.consign.dummy`）。
    async fn confirm_shipping(&self, order_id: &str) -> ConfirmResult;

    /// 发送文本到会话（ws 消息）。
    async fn send_text(&self, chat_id: &str, buyer_id: &str, text: &str) -> DingDaResult<()>;

    /// 更新订单发货信息（状态/方式/内容/失败原因）。
    async fn update_order_delivery(
        &self,
        order_no: &str,
        status: &str,
        delivery_method: &str,
        content: &str,
        fail_reason: &str,
    ) -> DingDaResult<()>;

    /// 按商品 ID 获取候选卡券（业务层从存储加载）。
    fn cards_for_item(&self, item_id: &str) -> Vec<Card>;

    /// data 类型卡券消费一条（行锁由存储实现保证）。
    fn consume_batch_data(&self, card_id: i64) -> Option<String>;

    /// api 类型卡券内容拉取。
    fn fetch_api_content(&self, api_config: &str) -> Option<String>;

    /// 标记已发货（防止重复处理）。
    fn mark_delivery_sent(&self, order_id: &str);
}
