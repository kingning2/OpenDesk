//! 渠道协调器 — 入站处理 + 自动回复决策 + 事件推送。
//!
//! 实现 [`InboundListener`]，在协议层与业务层之间编排：
//! 入站消息 → 去重/持久化 → 事件 → 自动回复（`AutoReplyPipeline`）→ 安全过滤 → 发送。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use agent::llm::ChatMessage;
use app::auto_reply::{AutoReplyDecision, ChatInput};
use app::risk::RiskService;
use app::xianyu::InMemoryRiskStore;
use common::contracts::{ChannelConversation, ChannelMessage, ChannelSettings};
use common::events::{emit, AppEvent, ChannelMessageEvent, ChannelStatusEvent, EventSink};
use common::{DingDaError, DingDaResult};
use platform::xianyu::{cookies, is_risk_control_text};
use serde_json::Value;

use crate::shared::auto_reply::AutoReplyHandle;

#[cfg(platform_xianyu)]
use super::cookie_renew::RiskCookieRenewer;
use super::dispatcher::ChannelDispatcher;
use super::protocol::{ChannelInboundMessage, ConnectionState, ConversationSync, InboundListener};
use super::{conversation_id_for, filter_reply, inbound_to_message, ChannelRepo};

/// 协调器 — 持有 store / dispatcher / 自动回复管线 / 事件总线。
pub struct ChannelCoordinator {
    store: Arc<ChannelRepo>,
    dispatcher: Arc<ChannelDispatcher>,
    auto_reply: AutoReplyHandle,
    sink: Arc<dyn EventSink>,
    risk_store: Option<Arc<InMemoryRiskStore>>,
    owner_id: i64,
    /// 风控日志去重：account_id → (detail 摘要, 毫秒时间戳)。
    risk_dedup: Mutex<HashMap<String, (String, u128)>>,
    /// 滑块验证浏览器续期（闲鱼启用时注入）。
    #[cfg(platform_xianyu)]
    cookie_renewer: Option<Arc<RiskCookieRenewer>>,
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
    /// - `risk_store` — 风控日志存储（闲鱼启用时注入）
    /// - `cookie_renewer` — 滑块浏览器续期（闲鱼启用时注入）
    pub fn new(
        store: Arc<ChannelRepo>,
        dispatcher: Arc<ChannelDispatcher>,
        auto_reply: AutoReplyHandle,
        sink: Arc<dyn EventSink>,
        risk_store: Option<Arc<InMemoryRiskStore>>,
        #[cfg(platform_xianyu)] cookie_renewer: Option<Arc<RiskCookieRenewer>>,
    ) -> Self {
        Self {
            store,
            dispatcher,
            auto_reply,
            sink,
            risk_store,
            owner_id: 1,
            risk_dedup: Mutex::new(HashMap::new()),
            #[cfg(platform_xianyu)]
            cookie_renewer,
        }
    }

    fn maybe_record_risk(&self, account_id: &str, detail: &str) {
        let Some(risk_store) = &self.risk_store else {
            return;
        };
        if !is_risk_control_text(detail) {
            return;
        }

        let signature = detail.chars().take(200).collect::<String>();
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        {
            let mut dedup = self
                .risk_dedup
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some((last_sig, last_ms)) = dedup.get(account_id) {
                if last_sig == &signature && now_ms.saturating_sub(*last_ms) < 120_000 {
                    return;
                }
            }
            dedup.insert(account_id.to_string(), (signature, now_ms));
        }

        let service = RiskService::new(risk_store.as_ref());
        match service.record_im_risk(self.owner_id, account_id, "闲鱼 IM", detail) {
            Ok(log) => {
                info!(
                    account = %account_id,
                    log_id = log.id,
                    "已写入风控日志"
                );
            }
            Err(error) => {
                warn!(account = %account_id, %error, "写入风控日志失败");
            }
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
        let cid = conversation
            .cid
            .clone()
            .unwrap_or_else(|| conversation.peer_id.clone());
        let message_id = self
            .dispatcher
            .send(
                &conversation.account_id,
                &cid,
                &conversation.peer_id,
                content,
            )
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

    /// 拉取会话完整消息历史并落库、推送；返回新插入条数。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-20
    pub async fn fetch_history(&self, conversation_id: &str) -> DingDaResult<usize> {
        let conversation = self
            .store
            .find_conversation_by_id(conversation_id)
            .map_err(|error| DingDaError::store(error.to_string()))?
            .ok_or_else(|| DingDaError::not_found("conversation", conversation_id))?;

        // 账号自身 goofish id（unb），用于判断消息方向（我发的 → out）。
        let my_unb = self
            .store
            .list_accounts()
            .map_err(|error| DingDaError::store(error.to_string()))?
            .into_iter()
            .find(|account| account.id == conversation.account_id)
            .and_then(|account| {
                let cookie_list = cookies::parse_credential(&account.credential);
                cookies::my_id(&cookie_list)
            });

        let cid = conversation
            .cid
            .clone()
            .unwrap_or_else(|| conversation.peer_id.clone());
        let history = self
            .dispatcher
            .fetch_history(&conversation.account_id, &cid)
            .await?;

        let existing = self
            .store
            .list_messages(&conversation.id)
            .map_err(|error| DingDaError::store(error.to_string()))?;

        let mut inserted = 0usize;
        for (index, item) in history.into_iter().enumerate() {
            let outbound = my_unb
                .as_ref()
                .is_some_and(|unb| item.sender_user_id == *unb);
            let message = ChannelMessage {
                id: format!("h-{}-{}-{}", conversation.id, item.created_at_ms, index),
                conversation_id: conversation.id.clone(),
                direction: if outbound {
                    "out".to_string()
                } else {
                    "in".to_string()
                },
                sender: if outbound {
                    "human".to_string()
                } else {
                    "customer".to_string()
                },
                content: item.content,
                created_at: item.created_at_ms.to_string(),
            };
            // 去重：同会话同时间同内容的已存在则跳过（WS 推送与历史可能重复）。
            if existing
                .iter()
                .any(|m| m.created_at == message.created_at && m.content == message.content)
            {
                continue;
            }
            self.store
                .insert_message(&message)
                .map_err(|error| DingDaError::store(error.to_string()))?;
            self.emit_channel_message(&conversation.account_id, message, None);
            inserted += 1;
        }
        info!(
            conversation_id = %conversation_id,
            inserted,
            "会话消息历史已同步"
        );
        Ok(inserted)
    }

    /// 拉取会话关联商品卡信息（`message.headinfo`，GET）。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-20
    pub async fn fetch_conversation_headinfo(&self, conversation_id: &str) -> DingDaResult<Value> {
        let conversation = self
            .store
            .find_conversation_by_id(conversation_id)
            .map_err(|error| DingDaError::store(error.to_string()))?
            .ok_or_else(|| DingDaError::not_found("conversation", conversation_id))?;
        let account = self
            .store
            .list_accounts()
            .map_err(|error| DingDaError::store(error.to_string()))?
            .into_iter()
            .find(|account| account.id == conversation.account_id)
            .ok_or_else(|| DingDaError::not_found("account", conversation.account_id))?;
        let cookie_str =
            cookies::cookies_to_string(&cookies::parse_credential(&account.credential));
        let item_id = conversation.item_id.unwrap_or_default();
        let session_id = conversation
            .cid
            .clone()
            .unwrap_or_else(|| conversation.peer_id.clone());
        platform::xianyu::fetch_message_headinfo(&cookie_str, &session_id, &item_id).await
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
        let message_created_at = if inbound.created_at_ms > 0 {
            inbound.created_at_ms.to_string()
        } else {
            now.clone()
        };
        let conversation = ChannelConversation {
            id: conversation_id.clone(),
            account_id: inbound.account_id.clone(),
            cid: if inbound.cid.is_empty() {
                None
            } else {
                Some(inbound.cid.clone())
            },
            peer_id: inbound.peer_id.clone(),
            peer_name: Some(inbound.peer_name.clone()),
            item_id: Some(inbound.item_id.clone()),
            item_title: None,
            item_price: None,
            updated_at: message_created_at.clone(),
        };
        if let Err(error) = self.store.upsert_conversation(&conversation) {
            warn!(%error, "更新会话失败");
        }

        let message = inbound_to_message(&inbound, &conversation_id, &message_created_at);
        let existing = self
            .store
            .list_messages(&conversation_id)
            .unwrap_or_default();
        if existing.iter().any(|item| item.id == message.id) {
            return;
        }
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
            .send(
                &inbound.account_id,
                &inbound.cid,
                &inbound.peer_id,
                &safe_reply,
            )
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
                if let Some(detail) = detail.as_deref() {
                    if is_risk_control_text(detail) {
                        #[cfg(platform_xianyu)]
                        if let Some(renewer) = &self.cookie_renewer {
                            warn!(account = %account_id, "检测到风控，调度浏览器自动过滑块");
                            renewer
                                .clone()
                                .spawn_renew(account_id.to_string(), detail.to_string());
                        } else {
                            warn!(account = %account_id, "滑块续期器未注入，无法自动过滑块");
                        }
                    }
                    self.maybe_record_risk(account_id, detail);
                }
            }
            ConnectionState::Connecting => {}
        }
        self.emit_channel_status(account_id, state, detail);
    }

    async fn on_conversation(&self, sync: ConversationSync) {
        let conversation = ChannelConversation {
            id: conversation_id_for(&sync.peer_id, &sync.item_id),
            account_id: sync.account_id.clone(),
            cid: if sync.cid.is_empty() {
                None
            } else {
                Some(sync.cid.clone())
            },
            peer_id: sync.peer_id,
            peer_name: None,
            item_id: if sync.item_id.is_empty() {
                None
            } else {
                Some(sync.item_id)
            },
            item_title: if sync.item_title.is_empty() {
                None
            } else {
                Some(sync.item_title)
            },
            item_price: None,
            updated_at: sync.updated_at,
        };
        if let Err(error) = self.store.upsert_conversation(&conversation) {
            warn!(%error, "同步会话失败");
        }
    }
}
