//! 闲鱼 WebSocket 长连接 — 收发消息核心。
//!
//! 协议流程（参考 XianyuAutoAgent 逆向）：
//! 1. `connect` 用 Cookie 握手建立 WS；
//! 2. 发 `/reg` 注册（携带 token）；
//! 3. 发 sync ack；
//! 4. 收帧循环：心跳响应 → 通用 ACK → `syncPushPackage`（base64+MessagePack 解码）→ 上抛文本消息；
//! 5. 心跳 15s；token 每 1h 刷新并重连。

use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

use super::api::XianyuApi;
use super::codec;
use super::message;
use crate::protocol::{
    ChannelAccount, ChannelError, ChannelInboundMessage, ChannelProtocol, ConnectionState,
    InboundListener,
};

use common::constants::xianyu;
use common::DingDaResult;

const WS_URL: &str = xianyu::WS_URL;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);
const TOKEN_REFRESH_INTERVAL: Duration = Duration::from_secs(3600);

/// 连接流类型（保留供未来扩展类型标注）。
#[allow(dead_code)]
type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// 判断是否为认证类错误：会话过期 / cookie 失效 / 未登录等，重试无意义，需重新登录。
///
/// 注意：必须**精确匹配**真正的会话过期类错误码；风控类（`FAIL_SYS_USER_VALIDATE` /
/// `RGV587` / punish / captcha）不是会话过期，由 [`super::risk::is_risk_control_text`]
/// 单独识别：协调器会拉起浏览器续期，本循环仅作退避兜底。
fn is_auth_error(error: &ChannelError) -> bool {
    let text = error.to_string();
    [
        "FAIL_SYS_SESSION_EXPIRED",
        "FAIL_SYS_TOKEN_EMPTY",
        "SESSION_EXPIRED",
        "TOKEN_EMPTY",
        "cookie 缺少",
        "未登录",
        "Session过期",
    ]
    .iter()
    .any(|keyword| text.contains(keyword))
}

/// 判断是否为闲鱼风控拦截（验证码 / 签名异常 / 频率限制）。
fn is_risk_control(error: &ChannelError) -> bool {
    super::risk::is_risk_control_text(&error.to_string())
}

/// 风控重试退避：30s → 60s → 120s → 240s → 封顶 300s。
fn risk_control_backoff_secs(failure: u32) -> u64 {
    let power = failure.saturating_sub(1).min(4);
    (30u64 * 2u64.pow(power)).min(300)
}

/// 内部可变状态 — 通过 `Arc` 与后台任务共享。
struct Inner {
    account: RwLock<Option<ChannelAccount>>,
    state: RwLock<ConnectionState>,
    listener: RwLock<Option<Arc<dyn InboundListener>>>,
    writer: tokio::sync::Mutex<Option<mpsc::Sender<String>>>,
}

impl Inner {
    /// 锁中毒时恢复（`PoisonError::into_inner`），不 panic。
    fn read_state(&self) -> ConnectionState {
        *self
            .state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn set_state(&self, state: ConnectionState, detail: Option<String>) {
        *self
            .state
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
        let account_id = self
            .account
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map(|account| account.id.clone())
            .unwrap_or_default();
        let listener = self
            .listener
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let Some(listener) = listener {
            tokio::spawn(async move {
                listener.on_state(&account_id, state, detail).await;
            });
        }
    }

    fn notify_message(&self, message: ChannelInboundMessage) {
        let listener = self
            .listener
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let Some(listener) = listener {
            tokio::spawn(async move {
                listener.on_message(message).await;
            });
        }
    }
}

/// 闲鱼渠道协议实现。
#[derive(Clone)]
pub struct XianyuChannel {
    inner: Arc<Inner>,
    stop_flag: Arc<AtomicBool>,
    task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl Default for XianyuChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl XianyuChannel {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                account: RwLock::new(None),
                state: RwLock::new(ConnectionState::Disconnected),
                listener: RwLock::new(None),
                writer: tokio::sync::Mutex::new(None),
            }),
            stop_flag: Arc::new(AtomicBool::new(false)),
            task: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// 连接主循环：握手 → 注册 → ack → 收发帧。
    /// `outbound` 为跨重连的发送队列接收端。
    async fn run(&self, outbound: &mut mpsc::Receiver<String>) -> Result<(), ChannelError> {
        let account = self
            .inner
            .account
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
            .ok_or_else(|| ChannelError::NotConnected("no account".into()))?;
        let cookie_str = super::cookies::cookies_to_string(&super::cookies::parse_credential(
            &account.credential,
        ));
        let api = XianyuApi::new(&cookie_str)
            .map_err(|error| ChannelError::Protocol(error.to_string()))?;
        let cookies = super::cookies::parse_credential(&account.credential);
        // 校验 unb 存在（协议发送依赖）。
        let unb = super::cookies::my_id(&cookies)
            .ok_or_else(|| ChannelError::Protocol("cookie 缺少 unb".into()))?;
        let device_id = super::cookies::device_id(&cookies)
            .ok_or_else(|| ChannelError::Protocol("cookie 缺少 unb".into()))?;

        info!(account = %account.id, unb = %unb, device = %device_id, "闲鱼开始连接：正在获取 mtop token");
        let token = api
            .fetch_token()
            .await
            .map_err(|error| ChannelError::Protocol(error.to_string()))?;
        info!(account = %account.id, "mtop token 获取成功，正在建立 WebSocket");

        let (mut sink, mut stream) = connect_async(WS_URL)
            .await
            .map_err(|error| ChannelError::Transport(format!("ws 连接失败: {error}")))?
            .0
            .split();

        self.inner
            .set_state(ConnectionState::Connected, Some("连接成功".into()));
        info!(account = %account.id, "WebSocket 已连接，正在注册设备监听");

        let reg = message::register_frame(&device_id, &token);
        sink.send(Message::Text(reg.to_string()))
            .await
            .map_err(|error| ChannelError::Transport(error.to_string()))?;
        let ack = message::sync_ack_frame();
        sink.send(Message::Text(ack.to_string()))
            .await
            .map_err(|error| ChannelError::Transport(error.to_string()))?;
        info!(account = %account.id, "设备注册帧与同步确认已发送，开始监听消息");

        self.sync_unread_sessions(&cookie_str, &account.id).await;

        let mut last_heartbeat = Instant::now();
        let last_token_refresh = Instant::now();

        while !self.stop_flag.load(Ordering::SeqCst) {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(500)) => {
                    if last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
                        let frame = message::heartbeat_frame();
                        if sink.send(Message::Text(frame.to_string())).await.is_err() {
                            return Ok(());
                        }
                        debug!(account = %account.id, "已发送心跳帧");
                        last_heartbeat = Instant::now();
                    }
                    if last_token_refresh.elapsed() >= TOKEN_REFRESH_INTERVAL {
                        info!(account = %account.id, "token 到期，触发重连刷新");
                        return Ok(()); // 触发重连以刷新 token
                    }
                }
                Some(frame) = outbound.recv() => {
                    if sink.send(Message::Text(frame)).await.is_err() {
                        return Ok(());
                    }
                }
                incoming = stream.next() => {
                    match incoming {
                        Some(Ok(Message::Text(text))) => {
                            self.handle_text_frame(&text).await;
                        }
                        Some(Ok(Message::Binary(bin))) => {
                            self.handle_binary_frame(&bin).await;
                        }
                        Some(Ok(_)) => {}
                        Some(Err(error)) => {
                            return Err(ChannelError::Transport(error.to_string()));
                        }
                        None => return Ok(()),
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_text_frame(&self, _text: &str) {
        // 心跳响应 / 通用控制帧为 JSON，业务消息走二进制 MessagePack。
    }

    async fn handle_binary_frame(&self, bin: &[u8]) {
        let parsed: Result<rmpv::Value, _> = rmpv::decode::read_value(&mut &bin[..]);
        let Ok(frame) = parsed else {
            return;
        };
        if !codec::is_chat_message(&frame) {
            return;
        }
        let Some(content) = codec::get_string(&frame, codec::MSG_CONTENT) else {
            return;
        };
        let peer_name = codec::get_string(&frame, codec::MSG_SENDER_NAME).unwrap_or_default();
        let peer_id = codec::get_string(&frame, codec::MSG_SENDER_ID).unwrap_or_default();
        let created_at_ms = codec::get_i64(&frame, codec::MSG_CREATE_TIME).unwrap_or(0);
        let url = codec::get_string(&frame, codec::MSG_URL).unwrap_or_default();
        let item_id = message::extract_item_id(&url).unwrap_or_default();
        let account_id = self
            .inner
            .account
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map(|account| account.id.clone())
            .unwrap_or_default();

        if content.is_empty() || peer_id.is_empty() {
            return;
        }

        info!(account = %account_id, peer = %peer_id, item = %item_id, "闲鱼收到入站消息帧");
        let inbound = ChannelInboundMessage {
            account_id,
            peer_id,
            peer_name,
            item_id,
            content,
            created_at_ms,
        };
        self.inner.notify_message(inbound);
    }

    /// 连接成功后拉取 mtop 未读会话，补全 WebSocket 推送前的待回复消息。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-20
    async fn sync_unread_sessions(&self, cookie_str: &str, account_id: &str) {
        match super::session::fetch_unread_sessions(cookie_str, 50).await {
            Ok((sessions, _)) => {
                if sessions.is_empty() {
                    info!(account = %account_id, "无未读会话");
                    return;
                }
                info!(
                    account = %account_id,
                    count = sessions.len(),
                    "已拉取未读会话，正在同步"
                );
                for session in sessions {
                    // 优先真人会话；系统/通知类仍保留有正文摘要的条目。
                    if session.session_type != 1 && session.last_msg.is_empty() {
                        continue;
                    }

                    let peer_id = if !session.peer_id.is_empty() {
                        session.peer_id
                    } else {
                        message::extract_cid(&session.session_id)
                    };
                    if peer_id.is_empty() {
                        continue;
                    }

                    let content = if session.last_msg.is_empty() {
                        format!("({} 条未读消息)", session.unread)
                    } else {
                        session.last_msg
                    };

                    let created_at_ms = if session.ts_ms > 0 {
                        session.ts_ms
                    } else {
                        message::now_ms()
                    };

                    let inbound = ChannelInboundMessage {
                        account_id: account_id.to_string(),
                        peer_id,
                        peer_name: session.peer_name,
                        item_id: session.item_id,
                        content,
                        created_at_ms,
                    };
                    self.inner.notify_message(inbound);
                }
            }
            Err(error) => {
                let detail = error.to_string();
                warn!(account = %account_id, %detail, "拉取未读会话失败");
                if super::risk::is_risk_control_text(&detail) {
                    self.inner.set_state(ConnectionState::Error, Some(detail));
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl ChannelProtocol for XianyuChannel {
    fn kind(&self) -> crate::protocol::ChannelKind {
        crate::protocol::ChannelKind::Xianyu
    }

    fn connection_state(&self) -> ConnectionState {
        self.inner
            .state
            .read()
            .map(|guard| *guard)
            .unwrap_or_else(|poisoned| *poisoned.into_inner())
    }

    fn active_account_id(&self) -> Option<String> {
        self.inner
            .account
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map(|account| account.id.clone())
    }

    fn set_inbound_listener(&self, listener: Arc<dyn InboundListener>) {
        *self
            .inner
            .listener
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(listener);
    }

    async fn connect(&self, account: &ChannelAccount) -> DingDaResult<()> {
        let current_account = self
            .inner
            .account
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let same_account = current_account
            .as_ref()
            .is_some_and(|current| current.id == account.id);

        if self.inner.read_state() == ConnectionState::Connected && same_account {
            return Ok(());
        }

        *self
            .inner
            .account
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(account.clone());

        self.inner
            .set_state(ConnectionState::Connecting, Some("正在连接".into()));

        self.stop_flag.store(false, Ordering::SeqCst);
        let (tx, mut rx) = mpsc::channel::<String>(64);
        *self.inner.writer.lock().await = Some(tx);

        // 终止旧的连接任务，避免重复连接循环。
        if let Some(old) = self.task.lock().await.take() {
            old.abort();
        }

        let this = self.clone();
        let stop_flag = self.stop_flag.clone();
        let account_id_for_log = account.id.clone();
        let task = tokio::spawn(async move {
            // 连续失败计数：风控时指数退避，成功（正常断开）后重置。
            let mut consecutive_failures: u32 = 0;
            loop {
                if stop_flag.load(Ordering::SeqCst) {
                    break;
                }
                let result = this.run(&mut rx).await;
                if stop_flag.load(Ordering::SeqCst) {
                    break;
                }
                if let Err(error) = result {
                    consecutive_failures += 1;
                    // 认证类错误（会话过期 / cookie 失效）重试无意义，停止自动重连，等待重新登录。
                    if is_auth_error(&error) {
                        error!(account = %account_id_for_log, %error, "闲鱼认证失效（会话过期 / cookie 无效），停止自动重连，请重新登录");
                        break;
                    }
                    // 风控拦截：非会话过期，指数退避后自动重试。
                    if is_risk_control(&error) {
                        let delay = risk_control_backoff_secs(consecutive_failures);
                        warn!(
                            account = %account_id_for_log,
                            %error,
                            retry_in_secs = delay,
                            "闲鱼风控拦截（验证码 / 签名异常），将尝试浏览器续期；失败则稍后自动重试"
                        );
                        this.inner
                            .set_state(ConnectionState::Error, Some(error.to_string()));
                        tokio::time::sleep(Duration::from_secs(delay)).await;
                        continue;
                    }
                    warn!(account = %account_id_for_log, %error, "闲鱼连接异常，5 秒后重连");
                    this.inner
                        .set_state(ConnectionState::Error, Some(error.to_string()));
                } else {
                    consecutive_failures = 0;
                    info!(account = %account_id_for_log, "闲鱼连接已断开，5 秒后重连");
                    this.inner.set_state(ConnectionState::Disconnected, None);
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        *self.task.lock().await = Some(task);
        Ok(())
    }

    async fn disconnect(&self) -> DingDaResult<()> {
        if let Some(account) = self
            .inner
            .account
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
        {
            info!(account = %account.id, "闲鱼主动断开连接");
        }
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(task) = self.task.lock().await.take() {
            task.abort();
        }
        self.inner.set_state(ConnectionState::Disconnected, None);
        Ok(())
    }

    async fn send(&self, peer_id: &str, text: &str) -> DingDaResult<String> {
        let account = self
            .inner
            .account
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
            .ok_or_else(|| ChannelError::NotConnected("no account".into()))?;
        let cookies = super::cookies::parse_credential(&account.credential);
        let my_id = super::cookies::my_id(&cookies)
            .ok_or_else(|| ChannelError::Protocol("cookie 缺少 unb".into()))?;
        let cid = message::extract_cid(peer_id);
        let frame = message::send_message_frame(&cid, &cid, &my_id, text);

        let sender = self
            .inner
            .writer
            .lock()
            .await
            .clone()
            .ok_or_else(|| ChannelError::NotConnected("ws 未连接".into()))?;
        sender
            .send(frame.to_string())
            .await
            .map_err(|error| ChannelError::Transport(error.to_string()))?;
        Ok(format!("xianyu-{}", message::generate_uuid()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proto(text: &str) -> ChannelError {
        ChannelError::Protocol(text.to_string())
    }

    #[test]
    fn auth_error_matches_real_session_expiry_only() {
        // 真正的会话过期 / cookie 失效。
        assert!(is_auth_error(&proto("FAIL_SYS_SESSION_EXPIRED")));
        assert!(is_auth_error(&proto("FAIL_SYS_TOKEN_EMPTY::缺少token")));
        assert!(is_auth_error(&proto("token 接口未成功: SESSION_EXPIRED")));
        assert!(is_auth_error(&proto("cookie 缺少 unb")));
        assert!(is_auth_error(&proto("未登录")));
        assert!(is_auth_error(&proto("Session过期")));
    }

    #[test]
    fn auth_error_does_not_match_risk_control() {
        // 风控拦截不是会话过期——这是本修复的核心回归用例。
        let punished = proto(
            r#"token 接口未成功: {"data":{"url":"...punish...captcha..."},"ret":["FAIL_SYS_USER_VALIDATE","RGV587_ERROR::SM::哎哟喂,被挤爆啦,请稍后重试"]}"#,
        );
        assert!(!is_auth_error(&punished), "风控不得被误判为认证失效");
        assert!(is_risk_control(&punished));
        // 其他非认证错误也不得误判。
        assert!(!is_auth_error(&proto(
            "mtop 请求失败 (api): connect timeout"
        )));
        assert!(!is_auth_error(&proto("ws 连接失败: Connection refused")));
    }

    #[test]
    fn risk_control_matches_punish_and_captcha() {
        assert!(is_risk_control(&proto("FAIL_SYS_USER_VALIDATE")));
        assert!(is_risk_control(&proto(
            "RGV587_ERROR::SM::哎哟喂,被挤爆啦,请稍后重试"
        )));
        assert!(is_risk_control(&proto(
            "punish?action=captcha&pureCaptcha="
        )));
        assert!(is_risk_control(&proto("FAIL_SYS_ILLEGAL_ACCESS")));
        assert!(!is_risk_control(&proto("FAIL_SYS_SESSION_EXPIRED")));
        assert!(!is_risk_control(&proto("ws 连接失败")));
    }

    #[test]
    fn risk_control_backoff_is_exponential_capped() {
        assert_eq!(risk_control_backoff_secs(1), 30);
        assert_eq!(risk_control_backoff_secs(2), 60);
        assert_eq!(risk_control_backoff_secs(3), 120);
        assert_eq!(risk_control_backoff_secs(4), 240);
        assert_eq!(risk_control_backoff_secs(5), 300);
        assert_eq!(risk_control_backoff_secs(99), 300);
    }
}
