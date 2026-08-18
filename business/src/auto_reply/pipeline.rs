//! 自动回复决策管线 — 分类 → 过滤 → 去重 → 关键词 → AI → 默认。
//!
//! 对齐 Python 版 auto_reply_service.handle_chat_message + get_reply：
//! 1. 卖家自己消息 → 暂停会话自动回复；
//! 2. 系统消息 / 发货触发 / 评价 / 确认收货 → 跳过自动回复（是否通知由上层决定）；
//! 3. 去重（会话+内容等待时间内不重复回复）；
//! 4. 过滤关键词（命中跳过回复）；
//! 5. 关键词匹配（商品 ID 优先）→ 命中即回；
//! 6. AI 回复（引擎生成）→ 成功即回；
//! 7. 默认回复（仅回复一次控制）。

use agent::knowledge::ItemKnowledge;
use agent::llm::ChatMessage;
use agent::reply::{AiSettings, ReplyContext, ReplyEngine, ReplyOutcome};

use super::classify::{MessageClass, MessageClassifier};
use super::dedup::{DedupKey, DedupStore};
use super::default::DefaultReplyStore;
use super::filter::KeywordFilter;
use super::keyword::{KeywordMatcher, KeywordRule};

/// 决策结果 — 上层（协调器）据此执行发送 / 通知 / 日志。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoReplyDecision {
    /// 无需回复（系统消息 / 自己消息 / 去重 / 过滤 / 无规则匹配）。
    Skip { reason: String },
    /// 关键词命中。
    Keyword(String),
    /// AI 生成。
    Ai(String),
    /// 默认回复。
    Default(String),
    /// 建议回复（AI 生成但未自动发送，供人工一键发出）。
    Suggestion(String),
}

/// 管线输出（含决策与上下文，供日志/通知复用）。
#[derive(Debug, Clone)]
pub struct AutoReplyOutcome {
    pub decision: AutoReplyDecision,
    /// 是否应发消息通知（未命中 skip_notify 过滤）。
    pub should_notify: bool,
}

/// 入站聊天消息上下文 — `handle_message` 的统一入参。
#[derive(Debug, Clone, Copy)]
pub struct ChatInput<'a> {
    pub account_id: &'a str,
    pub chat_id: &'a str,
    pub user_id: &'a str,
    pub item_id: Option<&'a str>,
    pub message: &'a str,
    /// 卖家自己发出的消息（暂停该会话自动回复）。
    pub sender_is_self: bool,
    /// 人工介入 / 暂停设置（上层判断）。
    pub pause_active: bool,
    /// AI 回复设置（None 表示未启用 AI）。
    pub ai_settings: Option<&'a AiSettings>,
    /// 商品知识（None 表示无商品信息，跳过 AI 步骤）。
    pub item: Option<&'a ItemKnowledge>,
    /// 对话历史（时间升序）。
    pub history: &'a [ChatMessage],
    /// 当前议价次数。
    pub bargain_count: u32,
}

impl<'a> ChatInput<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account_id: &'a str,
        chat_id: &'a str,
        user_id: &'a str,
        item_id: Option<&'a str>,
        message: &'a str,
        sender_is_self: bool,
        pause_active: bool,
        ai_settings: Option<&'a AiSettings>,
        item: Option<&'a ItemKnowledge>,
        history: &'a [ChatMessage],
        bargain_count: u32,
    ) -> Self {
        Self {
            account_id,
            chat_id,
            user_id,
            item_id,
            message,
            sender_is_self,
            pause_active,
            ai_settings,
            item,
            history,
            bargain_count,
        }
    }
}

/// 管线依赖 — 由业务层组装注入。
pub struct AutoReplyPipeline {
    pub filter: KeywordFilter,
    pub dedup: Box<dyn DedupStore>,
    pub keywords: KeywordMatcher,
    pub default_reply: Box<dyn DefaultReplyStore>,
}

impl AutoReplyPipeline {
    pub fn new(
        filter: KeywordFilter,
        dedup: Box<dyn DedupStore>,
        keywords: Vec<KeywordRule>,
        default_reply: Box<dyn DefaultReplyStore>,
    ) -> Self {
        Self {
            filter,
            dedup,
            keywords: KeywordMatcher::new(keywords),
            default_reply,
        }
    }

    /// 处理一条入站聊天消息。
    ///
    /// `sender_is_self` 为卖家自己消息（暂停自动回复）；
    /// `pause_active` 由上层判断（人工介入 / 暂停设置）传入。
    pub async fn handle_message(&self, input: &ChatInput<'_>) -> AutoReplyOutcome {
        let message = input.message;
        // 1. 分类：非聊天消息一律跳过自动回复。
        let class = MessageClassifier::classify(message, input.sender_is_self);
        if class != MessageClass::Chat {
            return self.notify_only(message, format!("class:{}", class_label(class)));
        }

        // 2. 暂停检查。
        if input.pause_active {
            return self.notify_only(message, "chat_paused".to_string());
        }

        // 3. 去重。
        let dedup_key = DedupKey::new(input.chat_id, message);
        if self.dedup.is_processed(&dedup_key) {
            tracing::info!(chat = %input.chat_id, "等待时间内已处理过，跳过重复回复");
            return self.notify_only(message, "duplicate_message".to_string());
        }

        // 4. 过滤。
        if self.filter.skip_reply(message) {
            return self.notify_only(message, "skip_reply_filter".to_string());
        }

        // 5. 关键词匹配。
        if let Some(matched) = self.keywords.match_message(message, input.item_id) {
            let reply = KeywordMatcher::render_reply(
                &matched,
                "",
                input.user_id,
                message,
                input.item_id.unwrap_or(""),
            );
            if reply.is_empty() {
                return self.notify_only(message, "empty_keyword_reply".to_string());
            }
            self.dedup.mark_processed(&dedup_key);
            tracing::info!(
                matched = %matched.matched_keyword,
                item = ?input.item_id,
                "关键词匹配成功"
            );
            return AutoReplyOutcome {
                decision: AutoReplyDecision::Keyword(reply),
                should_notify: !self.filter.skip_notify(message),
            };
        }

        // 6. AI 回复。
        if let (Some(settings), Some(item)) = (input.ai_settings, input.item) {
            let context = ReplyContext {
                user_message: message,
                item,
                history: input.history,
                bargain_count: input.bargain_count,
                settings,
            };
            match ReplyEngine::generate(&context).await {
                ReplyOutcome::Generated(reply) => {
                    self.dedup.mark_processed(&dedup_key);
                    return AutoReplyOutcome {
                        decision: AutoReplyDecision::Ai(reply),
                        should_notify: !self.filter.skip_notify(message),
                    };
                }
                ReplyOutcome::BargainLimited => {
                    self.dedup.mark_processed(&dedup_key);
                    return AutoReplyOutcome {
                        decision: AutoReplyDecision::Ai(
                            agent::reply::BARGAIN_LIMIT_REPLY.to_string(),
                        ),
                        should_notify: !self.filter.skip_notify(message),
                    };
                }
                ReplyOutcome::Disabled | ReplyOutcome::Failed(_) => {
                    // 回落默认回复。
                }
            }
        }

        // 7. 默认回复。
        if let Some((reply, once)) = self
            .default_reply
            .default_reply(input.account_id, input.item_id)
        {
            if !reply.trim().is_empty() {
                if once {
                    if self.default_reply.has_replied(
                        input.account_id,
                        input.user_id,
                        input.item_id,
                    ) {
                        return self.notify_only(message, "default_reply_once_used".to_string());
                    }
                    self.default_reply
                        .mark_replied(input.account_id, input.user_id, input.item_id);
                }
                self.dedup.mark_processed(&dedup_key);
                return AutoReplyOutcome {
                    decision: AutoReplyDecision::Default(reply),
                    should_notify: !self.filter.skip_notify(message),
                };
            }
        }

        // 8. 无规则匹配。
        self.notify_only(message, "no_rule_matched".to_string())
    }

    /// 跳过回复但仍可通知的公共出口。
    fn notify_only(&self, message: &str, reason: String) -> AutoReplyOutcome {
        AutoReplyOutcome {
            decision: AutoReplyDecision::Skip { reason },
            should_notify: !self.filter.skip_notify(message),
        }
    }
}

fn class_label(class: MessageClass) -> &'static str {
    match class {
        MessageClass::Chat => "chat",
        MessageClass::System => "system_message",
        MessageClass::AutoDeliveryTrigger => "auto_delivery_trigger",
        MessageClass::RateRequest => "rate_request",
        MessageClass::ConfirmReceipt => "confirm_receipt",
        MessageClass::SelfMessage => "self_message",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auto_reply::dedup::InMemoryDedup;
    use crate::auto_reply::default::InMemoryDefaultReply;
    use crate::auto_reply::filter::{FilterRule, FilterType, KeywordFilter};

    fn empty_ai() -> Option<AiSettings> {
        None
    }

    fn pipeline(keywords: Vec<KeywordRule>, default: InMemoryDefaultReply) -> AutoReplyPipeline {
        AutoReplyPipeline::new(
            KeywordFilter::new(vec![FilterRule {
                id: 0,
                account_id: String::new(),
                owner_id: 0,
                filter_type: FilterType::SkipReply,
                keyword: "勿扰".to_string(),
                enabled: true,
            }]),
            Box::new(InMemoryDedup::default()),
            keywords,
            Box::new(default),
        )
    }

    /// 构造测试入参。
    fn input<'a>(
        message: &'a str,
        sender_is_self: bool,
        ai: Option<&'a AiSettings>,
    ) -> ChatInput<'a> {
        ChatInput::new(
            "a",
            "c",
            "u",
            None,
            message,
            sender_is_self,
            false,
            ai,
            None,
            &[],
            0,
        )
    }

    #[tokio::test]
    async fn skips_system_message() {
        let pipe = pipeline(vec![], InMemoryDefaultReply::default());
        let outcome = pipe
            .handle_message(&input("[我已拍下，待付款]", false, empty_ai().as_ref()))
            .await;
        assert!(matches!(outcome.decision, AutoReplyDecision::Skip { .. }));
    }

    #[tokio::test]
    async fn self_message_skips() {
        let pipe = pipeline(vec![], InMemoryDefaultReply::default());
        let outcome = pipe
            .handle_message(&input("你好", true, empty_ai().as_ref()))
            .await;
        assert!(matches!(outcome.decision, AutoReplyDecision::Skip { .. }));
    }

    #[tokio::test]
    async fn filter_skips_reply() {
        let pipe = pipeline(vec![], InMemoryDefaultReply::default());
        let outcome = pipe
            .handle_message(&input("我现在勿扰", false, empty_ai().as_ref()))
            .await;
        assert!(matches!(outcome.decision, AutoReplyDecision::Skip { .. }));
    }

    #[tokio::test]
    async fn keyword_reply_wins() {
        let pipe = pipeline(
            vec![KeywordRule {
                id: 0,
                account_id: String::new(),
                keyword: "在吗".to_string(),
                reply: "在的".to_string(),
                item_id: String::new(),
                rule_type: "text".to_string(),
                image_url: String::new(),
                item_title: String::new(),
            }],
            InMemoryDefaultReply::default(),
        );
        let outcome = pipe
            .handle_message(&input("老板在吗", false, empty_ai().as_ref()))
            .await;
        assert_eq!(
            outcome.decision,
            AutoReplyDecision::Keyword("在的".to_string())
        );
    }

    #[tokio::test]
    async fn default_reply_fallback() {
        let mut default = InMemoryDefaultReply::default();
        default.add("a", "", "默认回复", false);
        let pipe = pipeline(vec![], default);
        let outcome = pipe
            .handle_message(&input("你好", false, empty_ai().as_ref()))
            .await;
        assert_eq!(
            outcome.decision,
            AutoReplyDecision::Default("默认回复".to_string())
        );
    }

    #[tokio::test]
    async fn no_rule_matched() {
        let pipe = pipeline(vec![], InMemoryDefaultReply::default());
        let outcome = pipe
            .handle_message(&input("你好", false, empty_ai().as_ref()))
            .await;
        assert!(matches!(outcome.decision, AutoReplyDecision::Skip { .. }));
    }

    #[tokio::test]
    async fn duplicate_message_skipped() {
        let pipe = pipeline(vec![], InMemoryDefaultReply::default());
        pipe.handle_message(&input("你好", false, empty_ai().as_ref()))
            .await;
        let outcome = pipe
            .handle_message(&input("你好", false, empty_ai().as_ref()))
            .await;
        assert!(matches!(outcome.decision, AutoReplyDecision::Skip { .. }));
    }
}
