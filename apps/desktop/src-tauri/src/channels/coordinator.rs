//! 渠道协调器 — 入站管线 + 自动回复决策 + 事件上抛。
//!
//! 实现 [`InboundListener`]，是协议层与业务层之间的桥：
//! 入站消息 → 归一化/持久化 → 事件 → 自动回复（规则路由 → LLM → 安全过滤 → 发送）→ 持久化 → 事件。

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use common::contracts::{
    ChannelConversation, ChannelEventMessage, ChannelMessage, ChannelSettings, LlmProvider,
};
use tauri::{AppHandle, Emitter};

use super::dispatcher::ChannelDispatcher;
use super::protocol::{ChannelInboundMessage, ConnectionState, InboundListener};
use super::reply::ReplyCoordinator;
use super::safety::filter_reply;
use super::store::{ChannelRepo, conversation_id_for, inbound_to_message};

/// 前端事件名。
pub const EVENT_CHANNEL_MESSAGE: &str = "channel.message";
pub const EVENT_CHANNEL_STATUS: &str = "channel.status";

/// 协调器 — 持有 store / dispatcher / reply / app 引用。
pub struct ChannelCoordinator {
    store: Arc<ChannelRepo>,
    dispatcher: Arc<ChannelDispatcher>,
    reply: Arc<ReplyCoordinator>,
    app: AppHandle,
}

impl ChannelCoordinator {
    pub fn new(
        store: Arc<ChannelRepo>,
        dispatcher: Arc<ChannelDispatcher>,
        reply: Arc<ReplyCoordinator>,
        app: AppHandle,
    ) -> Self {
        Self {
            store,
            dispatcher,
            reply,
            app,
        }
    }

    fn now_iso(&self) -> String {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        // 本地可读时间。
        format!("{millis}")
    }

    /// 人工发送：持久化出站消息 + 调协议发送 + 上抛事件。
    pub async fn send_message(
        &self,
        conversation: &ChannelConversation,
        content: &str,
    ) -> Result<String, String> {
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

        let event = ChannelEventMessage {
            account_id: conversation.account_id.clone(),
            message: outbound,
            suggestion: None,
        };
        self.emit_message(&event);
        Ok(message_id)
    }

    /// 查询 AI provider 配置（当前从 store 读取？无；由调用方注入）。
    /// 占位：此处不做 LLM 配置持久化，从调用方传入。
    #[allow(dead_code)]
    fn emit_status(&self, account_id: &str, state: ConnectionState, detail: Option<String>) {
        let _ = self.app.emit(
            EVENT_CHANNEL_STATUS,
            serde_json::json!({
                "account_id": account_id,
                "state": state.as_str(),
                "detail": detail,
            }),
        );
    }

    fn emit_message(&self, event: &ChannelEventMessage) {
        let _ = self.app.emit(EVENT_CHANNEL_MESSAGE, event);
    }
}

#[async_trait::async_trait]
impl InboundListener for ChannelCoordinator {
    async fn on_message(&self, inbound: ChannelInboundMessage) {
        tracing::info!(peer = %inbound.peer_id, item = %inbound.item_id, "channel inbound message");

        // 1. 归一化会话（peer+item 派生 id）。
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
            tracing::warn!(%error, "upsert conversation failed");
        }

        // 2. 持久化入站消息。
        let message = inbound_to_message(&inbound, &conversation_id, &now);
        if let Err(error) = self.store.insert_message(&message) {
            tracing::warn!(%error, "insert inbound message failed");
        }

        // 3. 上抛事件（UI 即时更新）。
        let event = ChannelEventMessage {
            account_id: inbound.account_id.clone(),
            message: message.clone(),
            suggestion: None,
        };
        self.emit_message(&event);

        // 4. 自动回复决策。
        let settings = self.store.get_settings().unwrap_or(ChannelSettings { auto_reply: false });
        if !settings.auto_reply {
            return;
        }

        // 4.1 组历史 + 调 LLM（provider 由上层注入；当前无全局 LLM 配置时跳过）。
        let history = self
            .store
            .list_messages(&conversation_id)
            .unwrap_or_default();
        let provider: Option<LlmProvider> = None; // TODO: 接入 AI 账号配置。
        let reply = match self
            .reply
            .generate_reply(&inbound.content, Some(&conversation), &history, provider.as_ref(), &inbound.account_id)
            .await
        {
            Ok(Some(response)) => response.reply,
            Ok(None) => {
                tracing::info!("intent no_reply, skip auto reply");
                return;
            }
            Err(error) => {
                tracing::warn!(%error, "llm reply generation failed");
                return;
            }
        };

        // 4.2 安全过滤（最后一道闸门）。
        let safe_reply = filter_reply(&reply);

        // 4.3 发送。
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
                    tracing::warn!(%error, "insert outbound message failed");
                }
                self.emit_message(&ChannelEventMessage {
                    account_id: inbound.account_id.clone(),
                    message: outbound,
                    suggestion: None,
                });
            }
            Err(error) => {
                tracing::warn!(%error, "auto reply send failed");
                // 发送失败：仅把建议上抛（UI 展示供人工一键发出）。
                self.emit_message(&ChannelEventMessage {
                    account_id: inbound.account_id.clone(),
                    message,
                    suggestion: Some(safe_reply),
                });
            }
        }
    }

    async fn on_state(&self, account_id: &str, state: ConnectionState, detail: Option<String>) {
        tracing::info!(account = %account_id, state = %state, "channel connection state changed");
        self.emit_status(account_id, state, detail);
    }
}
