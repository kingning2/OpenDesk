//! 渠道协调器 — 入站处理 + 自动回复决策 + 事件推送。
//!
//! 实现 [`InboundListener`]，在协议层与业务层之间编排：
//! 入站消息 → 去重/持久化 → 事件 → 自动回复（`AutoReplyPipeline`）→ 安全过滤 → 发送。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use agent::llm::ChatMessage;
use app::auto_reply::{AutoReplyDecision, ChatInput};
use common::contracts::{ChannelConversation, ChannelMessage, ChannelSettings};
use common::events::{emit, AppEvent, ChannelMessageEvent, ChannelStatusEvent, EventSink};
use common::DingDaResult;

use crate::shared::auto_reply::AutoReplyHandle;

use super::dispatcher::ChannelDispatcher;
use super::protocol::{ChannelInboundMessage, ConnectionState, InboundListener};
use super::{conversation_id_for, filter_reply, inbound_to_message, ChannelRepo};

/// 协调器 — 持有 store / dispatcher / 自动回复管线 / 事件总线。
pub struct ChannelCoordinator {
    store: Arc<ChannelRepo>,
    dispatcher: Arc<ChannelDispatcher>,
    auto_reply: AutoReplyHandle,
    sink: Arc<dyn EventSink>,
}

impl ChannelCoordinator {
    /// 创建协调器。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-18
    ///
    /// # 参数
    /// - `store` — 会话 / 消息持久化
    /// - `dispatcher` — 协议发送器
    /// - `auto_reply` — 自动回复管线
    /// - `sink` — 事件下发（`TauriEventSink` 或测试替身）
    pub fn new(
        store: Arc<ChannelRepo>,
        dispatcher: Arc<ChannelDispatcher>,
        auto_reply: AutoReplyHandle,
        sink: Arc<dyn EventSink>,
    ) -> Self {
        Self {
            store,
            dispatcher,
            auto_reply,
            sink,
        }
    }

    fn now_iso(&self) -> String {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        format!("{millis}")
    }

    /// 人工发送：持久化出站消息 + 经协议发送 + 推送事件。
    pub async fn send_message(
        &self,
        conversation: &ChannelConversation,
        content: &str,
    ) -> DingDaResult<String> {
        let message_id = self
            .dispatcher
            .send(&conversation.account_id, &conversation.peer_id, content)
            .await
            .map_err(|error| error.to_string())?;

        let outbound = ChannelMessage {
            id: format!("{}-out", message_id),
            conversation_id: conversation.id.clone(),
            direction: "out".to_string(),
            sender: "human".to_string(),
            content: content.to_string(),
            created_at: self.now_iso(),
        };
        self.store
            .insert_message(&outbound)
            .map_err(|error| error.to_string())?;

        self.emit_channel_message(&conversation.account_id, outbound, None);
        Ok(message_id)
    }

    fn emit_channel_status(
        &self,
        account_id: &str,
        state: ConnectionState,
        detail: Option<String>,
    ) {
        let event = AppEvent::ChannelStatus(ChannelStatusEvent {
            account_id: account_id.to_string(),
            state: state.as_str().to_string(),
            detail,
        });
        if let Err(e) = emit(self.sink.as_ref(), &event) {
            warn!(%e, "emit channel status failed");
        }
    }

    fn emit_channel_message(
        &self,
        account_id: &str,
        message: ChannelMessage,
        suggestion: Option<String>,
    ) {
        let event = AppEvent::ChannelMessage(ChannelMessageEvent {
            account_id: account_id.to_string(),
            message,
            suggestion,
        });
        if let Err(e) = emit(self.sink.as_ref(), &event) {
            warn!(%e, "emit channel message failed");
        }
    }

    fn to_chat_history(messages: &[ChannelMessage]) -> Vec<ChatMessage> {
        messages
            .iter()
            .map(|message| {
                let role = if message.direction == "out" {
                    "assistant"
                } else {
                    "user"
                };
                ChatMessage {
                    role: role.to_string(),
                    content: message.content.clone(),
                }
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl InboundListener for ChannelCoordinator {
    async fn on_message(&self, inbound: ChannelInboundMessage) {
        info!(peer = %inbound.peer_id, item = %inbound.item_id, "渠道收到入站消息");

        let conversation_id = conversation_id_for(&inbound.peer_id, &inbound.item_id);
        let now = self.now_iso();
        let conversation = ChannelConversation {
            id: conversation_id.clone(),
            account_id: inbound.account_id.clone(),
            peer_id: inbound.peer_id.clone(),
            peer_name: Some(inbound.peer_name.clone()),
            item_id: Some(inbound.item_id.clone()),
            item_title: None,
            item_price: None,
            updated_at: now.clone(),
        };
        if let Err(error) = self.store.upsert_conversation(&conversation) {
            warn!(%error, "更新会话失败");
        }

        let message = inbound_to_message(&inbound, &conversation_id, &now);
        if let Err(error) = self.store.insert_message(&message) {
            warn!(%error, "写入入站消息失败");
        }

        self.emit_channel_message(&inbound.account_id, message.clone(), None);

        let settings = self
            .store
            .get_settings()
            .unwrap_or(ChannelSettings { auto_reply: false });
        if !settings.auto_reply {
            return;
        }

        let history = self
            .store
            .list_messages(&conversation_id)
            .unwrap_or_default();
        let chat_history = Self::to_chat_history(&history);
        let input = ChatInput::new(
            &inbound.account_id,
            &conversation_id,
            &inbound.peer_id,
            Some(&inbound.item_id),
            &inbound.content,
            false,
            false,
            None,
            None,
            &chat_history,
            0,
        );

        let outcome = self.auto_reply.pipeline().handle_message(&input).await;
        let (reply_text, suggestion_only) = match outcome.decision {
            AutoReplyDecision::Skip { reason } => {
                info!(%reason, "跳过自动回复");
                return;
            }
            AutoReplyDecision::Suggestion(text) => (text, true),
            AutoReplyDecision::Keyword(text)
            | AutoReplyDecision::Ai(text)
            | AutoReplyDecision::Default(text) => (text, false),
        };

        let safe_reply = filter_reply(&reply_text);
        if suggestion_only {
            self.emit_channel_message(&inbound.account_id, message, Some(safe_reply));
            return;
        }

        match self
            .dispatcher
            .send(&inbound.account_id, &inbound.peer_id, &safe_reply)
            .await
        {
            Ok(message_id) => {
                let outbound = ChannelMessage {
                    id: format!("{}-out", message_id),
                    conversation_id: conversation_id.clone(),
                    direction: "out".to_string(),
                    sender: "ai".to_string(),
                    content: safe_reply.clone(),
                    created_at: self.now_iso(),
                };
                if let Err(error) = self.store.insert_message(&outbound) {
                    warn!(%error, "写入出站消息失败");
                }
                self.emit_channel_message(&inbound.account_id, outbound, None);
            }
            Err(error) => {
                warn!(%error, "自动回复发送失败");
                self.emit_channel_message(&inbound.account_id, message, Some(safe_reply));
            }
        }
    }

    async fn on_state(&self, account_id: &str, state: ConnectionState, detail: Option<String>) {
        match state {
            ConnectionState::Connected => {
                info!(account = %account_id, "已连接到闲鱼");
            }
            ConnectionState::Disconnected => {
                info!(account = %account_id, "已断开闲鱼连接");
            }
            ConnectionState::Error => {
                warn!(account = %account_id, detail = ?detail, "闲鱼连接异常");
            }
            ConnectionState::Connecting => {}
        }
        self.emit_channel_status(account_id, state, detail);
    }
}
