//! 渠道协调器 — 入站处理 + 事件推送。
//!
//! 实现 [`InboundListener`]，在协议层与业务层之间编排：
//! 入站消息 → 去重/持久化 → 事件推送。
//!
//! 平台无关：不引用 `platform_xianyu`；风控交给 [`super::risk_handler::RiskHandler`]。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use common::contracts::{ChannelConversation, ChannelMessage};
use common::events::{emit, AppEvent, ChannelMessageEvent, ChannelStatusEvent, EventSink};
use common::DingDaResult;

use super::dispatcher::ChannelDispatcher;
use super::protocol::{ChannelInboundMessage, ConnectionState, ConversationSync, InboundListener};
use super::risk_handler::RiskHandler;
use super::{conversation_id_for, inbound_to_message, ChannelRepo};

/// 登录态过期类错误（推 `auth_expired`，勿把原文 JSON 给前端）。
fn is_auth_expired_text(text: &str) -> bool {
    [
        "FAIL_SYS_SESSION_EXPIRED",
        "Session过期",
        "SESSION_EXPIRED",
        "登录态已过期",
        "请重新扫码登录",
        "cookie 缺少",
    ]
    .iter()
    .any(|keyword| text.contains(keyword))
}

/// 压缩错误 detail：禁止把 punish/token 整段 JSON 推到 UI。
fn sanitize_status_detail(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "连接异常，请稍后重试".into();
    }
    if trimmed.contains("_____tmd_____")
        || trimmed.contains("FAIL_SYS_USER_VALIDATE")
        || trimmed.contains("punish")
    {
        return "风控拦截，请稍后重试".into();
    }
    if trimmed.starts_with('{') || trimmed.len() > 120 {
        return "连接异常，请稍后重试或查看运行日志".into();
    }
    trimmed.chars().take(120).collect()
}

/// 协调器 — 持有 store / dispatcher / 事件总线 / 可选风控处理。
pub struct ChannelCoordinator {
    store: Arc<ChannelRepo>,
    dispatcher: Arc<ChannelDispatcher>,
    sink: Arc<dyn EventSink>,
    /// 平台风控处理（闲鱼启用时注入）；`None` 时风控错误退化为通用 error。
    risk_handler: Option<Arc<dyn RiskHandler>>,
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
    /// - `sink` — 事件下发（`TauriEventSink` 或测试替身）
    /// - `risk_handler` — 平台风控处理（闲鱼启用时注入）
    pub fn new(
        store: Arc<ChannelRepo>,
        dispatcher: Arc<ChannelDispatcher>,
        sink: Arc<dyn EventSink>,
        risk_handler: Option<Arc<dyn RiskHandler>>,
    ) -> Self {
        Self {
            store,
            dispatcher,
            sink,
            risk_handler,
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

    fn emit_channel_status(&self, account_id: &str, state: &str, detail: Option<String>) {
        let event = AppEvent::ChannelStatus(ChannelStatusEvent {
            account_id: account_id.to_string(),
            state: state.to_string(),
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
}

#[async_trait::async_trait]
impl InboundListener for ChannelCoordinator {
    async fn on_message(&self, inbound: ChannelInboundMessage) {
        info!(peer = %inbound.peer_id, item = %inbound.item_id, "渠道收到入站消息");

        // 优先按 cid 合并：WS 推包可能先用 cid 占位建会话，真消息到达后复用同一行。
        let conversation_id = if !inbound.cid.is_empty() {
            match self.store.find_conversation_by_cid(&inbound.cid) {
                Ok(Some(existing)) => existing.id,
                Ok(None) => conversation_id_for(&inbound.peer_id, &inbound.item_id),
                Err(error) => {
                    warn!(%error, "按 cid 查会话失败");
                    conversation_id_for(&inbound.peer_id, &inbound.item_id)
                }
            }
        } else {
            conversation_id_for(&inbound.peer_id, &inbound.item_id)
        };
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

        self.emit_channel_message(&inbound.account_id, message, None);
    }

    async fn on_state(&self, account_id: &str, state: ConnectionState, detail: Option<String>) {
        match state {
            ConnectionState::Connected => {
                info!(account = %account_id, "已连接到闲鱼");
                self.emit_channel_status(account_id, "connected", None);
            }
            ConnectionState::Disconnected => {
                info!(account = %account_id, "已断开闲鱼连接");
                self.emit_channel_status(account_id, "disconnected", None);
            }
            ConnectionState::Connecting => {
                self.emit_channel_status(account_id, "connecting", Some("正在连接闲鱼…".into()));
            }
            ConnectionState::Error => {
                warn!(account = %account_id, detail = ?detail, "闲鱼连接异常");
                let detail_text = detail.as_deref().unwrap_or("");
                if is_auth_expired_text(detail_text) {
                    self.emit_channel_status(
                        account_id,
                        "auth_expired",
                        Some("登录态已过期，请重新扫码后再连接".into()),
                    );
                    return;
                }
                // 平台风控处理：命中则记录日志并消费本次错误（避免把原文推给前端）。
                if let Some(risk_handler) = &self.risk_handler {
                    if risk_handler.is_risk_control_text(detail_text) {
                        risk_handler.record_risk(account_id, detail_text);
                        if risk_handler.handle_risk(account_id, detail_text) {
                            return;
                        }
                    }
                }
                self.emit_channel_status(
                    account_id,
                    "error",
                    Some(sanitize_status_detail(detail_text)),
                );
            }
        }
    }

    async fn on_conversation(&self, sync: ConversationSync) {
        // 已有同 cid 会话则复用 id，避免 watch 占位 peer 与 baseline 真 peer 拆成两行。
        let conversation_id = if !sync.cid.is_empty() {
            match self.store.find_conversation_by_cid(&sync.cid) {
                Ok(Some(existing)) => existing.id,
                Ok(None) => conversation_id_for(&sync.peer_id, &sync.item_id),
                Err(error) => {
                    warn!(%error, "按 cid 查会话失败");
                    conversation_id_for(&sync.peer_id, &sync.item_id)
                }
            }
        } else {
            conversation_id_for(&sync.peer_id, &sync.item_id)
        };

        let existing = self
            .store
            .find_conversation_by_id(&conversation_id)
            .ok()
            .flatten();
        // 占位 peer（=cid）不覆盖已有真实 peer。
        let peer_id = match &existing {
            Some(row) if row.peer_id != sync.cid && sync.peer_id == sync.cid => row.peer_id.clone(),
            _ => sync.peer_id.clone(),
        };
        let peer_name = existing.as_ref().and_then(|row| row.peer_name.clone());
        let item_id = if sync.item_id.is_empty() {
            existing.as_ref().and_then(|row| row.item_id.clone())
        } else {
            Some(sync.item_id.clone())
        };
        let item_title = if sync.item_title.is_empty() {
            existing.as_ref().and_then(|row| row.item_title.clone())
        } else {
            Some(sync.item_title.clone())
        };

        let conversation = ChannelConversation {
            id: conversation_id,
            account_id: sync.account_id.clone(),
            cid: if sync.cid.is_empty() {
                None
            } else {
                Some(sync.cid.clone())
            },
            peer_id,
            peer_name,
            item_id,
            item_title,
            item_price: existing.as_ref().and_then(|row| row.item_price),
            updated_at: sync.updated_at,
        };
        if let Err(error) = self.store.upsert_conversation(&conversation) {
            warn!(%error, "同步会话失败");
        }
    }
}
