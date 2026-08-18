//! 消息分类 — 决定消息走哪条处理链。
//!
//! 对齐 Python 版 auto_reply_service 的分类逻辑：
//! - 系统消息：跳过自动回复（但仍可通知）；
//! - 自动发货触发：交给发货流程（本阶段仅标记，发货后续实现）；
//! - 评价请求 / 确认收货：单独处理（后续实现）；
//! - 普通聊天：走自动回复决策链。

/// 消息分类结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageClass {
    /// 普通聊天消息 → 走自动回复。
    Chat,
    /// 系统消息（无需回复，可通知）。
    System,
    /// 自动发货触发消息（交发货流程）。
    AutoDeliveryTrigger,
    /// 评价请求消息（交评价流程）。
    RateRequest,
    /// 确认收货消息（交评价/通知流程）。
    ConfirmReceipt,
    /// 卖家自己发出的消息（暂停会话自动回复）。
    SelfMessage,
}

/// 系统消息关键词（跳过自动回复）。
const SYSTEM_MESSAGES_TO_SKIP: &[&str] = &[
    "[我已拍下，待付款]",
    "[你关闭了订单，钱款已原路退返]",
    "[不想宝贝被砍价?设置不砍价回复  ]",
    "AI正在帮你回复消息，不错过每笔订单",
    "发来一条消息",
    "发来一条新消息",
    "卖家人不错？送Ta闲鱼小红花",
    "你人真不错，送你闲鱼小红花",
    "[你已确认收货，交易成功]",
    "[买家确认收货，交易成功]",
    "买家已确认收货，交易成功",
    "[你已发货]",
    "已发货",
    "[注意！小心假客服骗钱！]",
    "「我将「退货退款」修改为「退款」」",
    "订单已签收",
    "有蚂蚁森林能量可领",
    "[我完成了评价]",
    "我完成了评价",
    "[退款成功，钱款已原路退返]",
    "[买家申请退款]",
    "[卖家同意退款]",
    "温馨提醒：商品信息近期有过变更",
    "查看商品详情",
];

/// 自动发货触发关键词。
const AUTO_DELIVERY_KEYWORDS: &[&str] = &[
    "[我已付款，等待你发货]",
    "[已付款，待发货]",
    "我已付款，等待你发货",
    "[记得及时发货]",
];

/// 评价请求关键词。
const RATE_REQUEST_KEYWORDS: &[&str] = &["快给ta一个评价吧", "快给TA一个评价吧", "给个评价"];

/// 确认收货关键词。
const CONFIRM_RECEIPT_KEYWORDS: &[&str] = &["确认收货", "交易成功"];

/// 消息分类器（纯函数，可单测）。
pub struct MessageClassifier;

impl MessageClassifier {
    pub fn classify(content: &str, sender_is_self: bool) -> MessageClass {
        if sender_is_self {
            return MessageClass::SelfMessage;
        }
        if SYSTEM_MESSAGES_TO_SKIP
            .iter()
            .any(|kw| content.contains(kw))
        {
            return MessageClass::System;
        }
        if AUTO_DELIVERY_KEYWORDS.iter().any(|kw| content.contains(kw)) {
            return MessageClass::AutoDeliveryTrigger;
        }
        if RATE_REQUEST_KEYWORDS.iter().any(|kw| content.contains(kw)) {
            return MessageClass::RateRequest;
        }
        if CONFIRM_RECEIPT_KEYWORDS
            .iter()
            .any(|kw| content.contains(kw))
        {
            return MessageClass::ConfirmReceipt;
        }
        MessageClass::Chat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_system_messages() {
        assert_eq!(
            MessageClassifier::classify("[我已拍下，待付款]", false),
            MessageClass::System
        );
        assert_eq!(
            MessageClassifier::classify("有蚂蚁森林能量可领", false),
            MessageClass::System
        );
    }

    #[test]
    fn classifies_delivery_trigger() {
        assert_eq!(
            MessageClassifier::classify("[我已付款，等待你发货]", false),
            MessageClass::AutoDeliveryTrigger
        );
    }

    #[test]
    fn classifies_rate_and_receipt() {
        assert_eq!(
            MessageClassifier::classify("快给ta一个评价吧~", false),
            MessageClass::RateRequest
        );
        // 确认收货文本在系统消息列表中（跳过自动回复，评价流程单独触发）。
        assert_eq!(
            MessageClassifier::classify("买家已确认收货，交易成功", false),
            MessageClass::System
        );
        // 非系统列表的确认收货文本 → ConfirmReceipt。
        assert_eq!(
            MessageClassifier::classify("麻烦确认收货", false),
            MessageClass::ConfirmReceipt
        );
    }

    #[test]
    fn classifies_self_and_chat() {
        assert_eq!(
            MessageClassifier::classify("你好", true),
            MessageClass::SelfMessage
        );
        assert_eq!(
            MessageClassifier::classify("你好", false),
            MessageClass::Chat
        );
        assert_eq!(
            MessageClassifier::classify("多少钱", false),
            MessageClass::Chat
        );
    }
}
