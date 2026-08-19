//! 发货执行器 — 卡券匹配 → 内容生成 → 确认发货 → 发送。
//!
//! 对齐 Python 版 `_auto_delivery` 核心流程（边界从简）：
//! 1. 商品 ID 无效 → 失败；
//! 2. 按商品取卡券 → 来源优先级唯一匹配（无匹配 → 失败）；
//! 3. card_only + 对接卡券 → 跳过（避免财务损失）；
//! 4. 延时；
//! 5. 确认发货（开关/冷却/幂等；只发卡券模式跳过）；
//! 6. 生成内容并发送；更新订单状态。

use super::card::CardSelector;
use super::content::{ContentContext, ContentGenerator};
use super::gateway::DeliveryGateway;

/// 发货执行开关（账号级设置，业务层注入）。
#[derive(Debug, Clone, Default)]
pub struct DeliveryOptions {
    /// 是否自动确认发货。
    pub auto_confirm: bool,
    /// 只发卡券（跳过确认发货，直接发卡）。
    pub only_send_card: bool,
    /// card_only 场景（禁止发货 + 关单 + 只发卡券）。
    pub closed_order_card_only: bool,
    /// 已确认订单冷却时间（秒，防重复确认）。
    pub confirm_cooldown_secs: u64,
}

/// 发货请求。
#[derive(Debug, Clone)]
pub struct DeliveryRequest {
    pub item_id: String,
    pub order_id: Option<String>,
    pub chat_id: Option<String>,
    pub buyer_id: String,
    pub buyer_name: String,
}

/// 发货结果。
#[derive(Debug, Clone)]
pub struct DeliveryResult {
    pub success: bool,
    /// 失败原因（成功为空）。
    pub fail_reason: String,
    /// 已发货过（幂等跳过）。
    pub already_delivered: bool,
    /// 发送的内容（成功时）。
    pub sent_content: Option<String>,
}

impl DeliveryResult {
    fn failure(reason: impl Into<String>) -> Self {
        Self {
            success: false,
            fail_reason: reason.into(),
            already_delivered: false,
            sent_content: None,
        }
    }

    fn success(content: String) -> Self {
        Self {
            success: true,
            fail_reason: String::new(),
            already_delivered: false,
            sent_content: Some(content),
        }
    }

    fn already_delivered() -> Self {
        Self {
            success: false,
            fail_reason: "订单已发货过，不再发送卡券".to_string(),
            already_delivered: true,
            sent_content: None,
        }
    }
}

/// 发货执行器。
pub struct DeliveryExecutor<'a> {
    gateway: &'a dyn DeliveryGateway,
    options: &'a DeliveryOptions,
}

impl<'a> DeliveryExecutor<'a> {
    pub fn new(gateway: &'a dyn DeliveryGateway, options: &'a DeliveryOptions) -> Self {
        Self { gateway, options }
    }

    /// 执行发货。
    pub async fn deliver(&self, request: &DeliveryRequest) -> DeliveryResult {
        // 1. 商品 ID 校验。
        if request.item_id.is_empty() || request.item_id == "未知商品" {
            return DeliveryResult::failure(format!(
                "商品ID无效，无法自动发货: {}",
                request.item_id
            ));
        }

        // 2. 卡券匹配。
        let cards = self.gateway.cards_for_item(&request.item_id);
        let Some(card) = CardSelector::select(&cards) else {
            return DeliveryResult::failure(format!(
                "商品 {} 未在任何来源中唯一匹配到卡券（共 {} 条关联）",
                request.item_id,
                cards.len()
            ));
        };

        // 3. card_only + 对接卡券 → 跳过。
        if self.options.closed_order_card_only && card.is_dock() {
            return DeliveryResult::failure(
                "card_only 模式不适用于对接卡券，为避免货主财务损失，跳过卡券发送",
            );
        }

        // 4. 延时。
        if card.delay_seconds > 0 {
            info!(card_id = card.id, delay = card.delay_seconds, "发货延时");
            tokio::time::sleep(std::time::Duration::from_secs(card.delay_seconds as u64)).await;
        }

        // 5. 确认发货。
        let confirm = match &request.order_id {
            Some(order_id) if !self.options.only_send_card => {
                if self.options.auto_confirm {
                    let result = self.gateway.confirm_shipping(order_id).await;
                    if result.already_delivered {
                        self.gateway.mark_delivery_sent(order_id);
                        self.gateway
                            .update_order_delivery(
                                order_id,
                                "shipped",
                                "auto",
                                "订单已确认发货（闲鱼平台已发货）",
                                "",
                            )
                            .await
                            .ok();
                        return DeliveryResult::already_delivered();
                    }
                    if !result.success {
                        warn!(order_id, message = %result.message, "确认发货失败");
                        return DeliveryResult::failure(format!(
                            "确认发货失败: {}",
                            result.message
                        ));
                    }
                    info!(order_id, "确认发货成功");
                    Some(result)
                } else {
                    None
                }
            }
            _ => None,
        };
        let _ = confirm;

        // 6. 生成内容并发送。
        let context = ContentContext {
            order_id: request.order_id.clone().unwrap_or_default(),
            item_id: request.item_id.clone(),
            buyer_id: request.buyer_id.clone(),
            ..Default::default()
        };
        let content = ContentGenerator::generate_with_api(
            card,
            &context,
            || self.gateway.consume_batch_data(card.id),
            |config| self.gateway.fetch_api_content(config),
        )
        .await;
        let Some(content) = content else {
            return DeliveryResult::failure(format!("商品 {} 卡券内容生成失败", request.item_id));
        };

        let text = content.to_text();
        if let (Some(chat_id), Some(order_id)) = (&request.chat_id, &request.order_id) {
            if let Err(error) = self
                .gateway
                .send_text(chat_id, &request.buyer_id, &text)
                .await
            {
                self.gateway
                    .update_order_delivery(
                        order_id,
                        "pending",
                        "auto",
                        &text,
                        &format!("发送失败: {error}"),
                    )
                    .await
                    .ok();
                return DeliveryResult::failure(format!("发送失败: {error}"));
            }
            self.gateway
                .update_order_delivery(order_id, "shipped", "auto", &text, "")
                .await
                .ok();
        }

        info!(
            item_id = %request.item_id,
            card_id = card.id,
            card_type = %card.card_type,
            "自动发货成功"
        );
        DeliveryResult::success(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delivery::execution::card::{Card, CardSource};
    use crate::delivery::execution::gateway::ConfirmResult;
    use common::DingDaResult;
    use std::sync::Mutex;

    struct MockGateway {
        cards: Vec<Card>,
        confirm_ok: bool,
        already: bool,
        sent: Mutex<Vec<String>>,
        order_updated: Mutex<Vec<String>>,
    }

    #[async_trait::async_trait]
    impl DeliveryGateway for MockGateway {
        async fn confirm_shipping(&self, _order_id: &str) -> ConfirmResult {
            ConfirmResult {
                success: self.confirm_ok,
                already_delivered: self.already,
                skipped_only_send_card: false,
                message: "ok".to_string(),
            }
        }
        async fn send_text(&self, _chat_id: &str, _buyer_id: &str, text: &str) -> DingDaResult<()> {
            self.sent.lock().expect("sent lock").push(text.to_string());
            Ok(())
        }
        async fn update_order_delivery(
            &self,
            order_no: &str,
            _status: &str,
            _delivery_method: &str,
            _content: &str,
            _fail_reason: &str,
        ) -> DingDaResult<()> {
            self.order_updated
                .lock()
                .expect("order lock")
                .push(order_no.to_string());
            Ok(())
        }
        fn cards_for_item(&self, _item_id: &str) -> Vec<Card> {
            self.cards.clone()
        }
        fn consume_batch_data(&self, _card_id: i64) -> Option<String> {
            Some("data1".to_string())
        }
        fn fetch_api_content(&self, _api_config: &str) -> Option<String> {
            Some("API卡密".to_string())
        }
        fn mark_delivery_sent(&self, _order_id: &str) {}
    }

    fn text_card() -> Card {
        Card {
            id: 1,
            owner_id: 0,
            account_id: String::new(),
            name: "卡".to_string(),
            card_type: "text".to_string(),
            source: CardSource::Own,
            enabled: true,
            text_content: "卡密123".to_string(),
            data_content: String::new(),
            image_url: String::new(),
            image_urls: String::new(),
            api_config: String::new(),
            delay_seconds: 0,
            description: String::new(),
        }
    }

    fn options() -> DeliveryOptions {
        DeliveryOptions {
            auto_confirm: true,
            only_send_card: false,
            closed_order_card_only: false,
            confirm_cooldown_secs: 0,
        }
    }

    fn mock_gateway(cards: Vec<Card>, confirm_ok: bool, already: bool) -> MockGateway {
        MockGateway {
            cards,
            confirm_ok,
            already,
            sent: Mutex::new(vec![]),
            order_updated: Mutex::new(vec![]),
        }
    }

    #[tokio::test]
    async fn invalid_item_fails() {
        let gateway = mock_gateway(vec![], true, false);
        let opts = options();
        let executor = DeliveryExecutor::new(&gateway, &opts);
        let result = executor
            .deliver(&DeliveryRequest {
                item_id: "未知商品".to_string(),
                order_id: None,
                chat_id: None,
                buyer_id: "b".to_string(),
                buyer_name: String::new(),
            })
            .await;
        assert!(!result.success);
    }

    #[tokio::test]
    async fn no_card_fails() {
        let gateway = mock_gateway(vec![], true, false);
        let opts = options();
        let executor = DeliveryExecutor::new(&gateway, &opts);
        let result = executor
            .deliver(&DeliveryRequest {
                item_id: "item-1".to_string(),
                order_id: Some("o-1".to_string()),
                chat_id: Some("c-1".to_string()),
                buyer_id: "b".to_string(),
                buyer_name: String::new(),
            })
            .await;
        assert!(!result.success);
        assert!(result.fail_reason.contains("唯一匹配"));
    }

    #[tokio::test]
    async fn delivers_text_card_and_updates_order() {
        let gateway = mock_gateway(vec![text_card()], true, false);
        let opts = options();
        let executor = DeliveryExecutor::new(&gateway, &opts);
        let result = executor
            .deliver(&DeliveryRequest {
                item_id: "item-1".to_string(),
                order_id: Some("o-1".to_string()),
                chat_id: Some("c-1".to_string()),
                buyer_id: "b".to_string(),
                buyer_name: String::new(),
            })
            .await;
        assert!(result.success, "reason: {}", result.fail_reason);
        assert_eq!(result.sent_content.as_deref(), Some("卡密123"));
        assert_eq!(gateway.sent.lock().expect("lock").len(), 1);
        assert!(gateway
            .order_updated
            .lock()
            .expect("lock")
            .contains(&"o-1".to_string()));
    }

    #[tokio::test]
    async fn already_delivered_skips() {
        let gateway = mock_gateway(vec![text_card()], true, true);
        let opts = options();
        let executor = DeliveryExecutor::new(&gateway, &opts);
        let result = executor
            .deliver(&DeliveryRequest {
                item_id: "item-1".to_string(),
                order_id: Some("o-1".to_string()),
                chat_id: Some("c-1".to_string()),
                buyer_id: "b".to_string(),
                buyer_name: String::new(),
            })
            .await;
        assert!(!result.success);
        assert!(result.already_delivered);
        assert!(gateway.sent.lock().expect("lock").is_empty());
    }

    #[tokio::test]
    async fn card_only_dock_skipped() {
        let mut dock = text_card();
        dock.source = CardSource::DockL1;
        let gateway = mock_gateway(vec![dock], true, false);
        let opts = DeliveryOptions {
            closed_order_card_only: true,
            ..options()
        };
        let executor = DeliveryExecutor::new(&gateway, &opts);
        let result = executor
            .deliver(&DeliveryRequest {
                item_id: "item-1".to_string(),
                order_id: Some("o-1".to_string()),
                chat_id: Some("c-1".to_string()),
                buyer_id: "b".to_string(),
                buyer_name: String::new(),
            })
            .await;
        assert!(!result.success);
        assert!(gateway.sent.lock().expect("lock").is_empty());
    }
}
