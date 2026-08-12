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
use crate::channels::protocol::{
    ChannelAccount, ChannelError, ChannelInboundMessage, ChannelProtocol, ConnectionState,
    InboundListener,
};

const WS_URL: &str = "wss://wss-goofish.dingtalk.com/";
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);
const TOKEN_REFRESH_INTERVAL: Duration = Duration::from_secs(3600);

/// 连接流类型（保留供未来扩展类型标注）。
#[allow(dead_code)]
type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// 判断是否为认证类错误：会话过期 / cookie 失效 / 未登录等，重试无意义，需重新登录。
fn is_auth_error(error: &ChannelError) -> bool {
    let text = error.to_string();
    [
        "SESSION_EXPIRED",
        "TOKEN_EMPTY",
        "FAIL_SYS",
        "cookie 缺少",
        "未登录",
        "Session过期",
    ]
    .iter()
    .any(|keyword| text.contains(keyword))
}

/// 内部可变状态 — 通过 `Arc` 与后台任务共享。
struct Inner {
    account: RwLock<Option<ChannelAccount>>,
    state: RwLock<ConnectionState>,
    listener: RwLock<Option<Arc<dyn InboundListener>>>,
    writer: tokio::sync::Mutex<Option<mpsc::Sender<String>>>,
}

impl Inner {
    fn set_state(&self, state: ConnectionState, detail: Option<String>) {
        *self.state.write().expect("state lock") = state;
        let account_id = self
            .account
            .read()
            .expect("account lock")
            .as_ref()
            .map(|account| account.id.clone())
            .unwrap_or_default();
        let listener = self.listener.read().expect("listener lock").clone();
        if let Some(listener) = listener {
            tokio::spawn(async move {
                listener.on_state(&account_id, state, detail).await;
            });
        }
    }

    fn notify_message(&self, message: ChannelInboundMessage) {
        let listener = self.listener.read().expect("listener lock").clone();
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
            .expect("account lock")
            .clone()
            .ok_or_else(|| ChannelError::NotConnected("no account".into()))?;
        let cookie_str = super::cookies::cookies_to_string(&super::cookies::parse_credential(
            &account.credential,
        ));
        let api = XianyuApi::new(&cookie_str).map_err(ChannelError::Protocol)?;
        let cookies = super::cookies::parse_credential(&account.credential);
        // 校验 unb 存在（协议发送依赖）。
        super::cookies::my_id(&cookies)
            .ok_or_else(|| ChannelError::Protocol("cookie 缺少 unb".into()))?;
        let device_id = super::cookies::device_id(&cookies)
            .ok_or_else(|| ChannelError::Protocol("cookie 缺少 unb".into()))?;

        let token = api.fetch_token().await.map_err(ChannelError::Protocol)?;

        let (mut sink, mut stream) = connect_async(WS_URL)
            .await
            .map_err(|error| ChannelError::Transport(format!("ws 连接失败: {error}")))?
            .0
            .split();

        self.inner
            .set_state(ConnectionState::Connected, Some("连接成功".into()));

        let reg = message::register_frame(&device_id, &token);
        sink.send(Message::Text(reg.to_string()))
            .await
            .map_err(|error| ChannelError::Transport(error.to_string()))?;
        let ack = message::sync_ack_frame();
        sink.send(Message::Text(ack.to_string()))
            .await
            .map_err(|error| ChannelError::Transport(error.to_string()))?;

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
                        last_heartbeat = Instant::now();
                    }
                    if last_token_refresh.elapsed() >= TOKEN_REFRESH_INTERVAL {
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
            .expect("account lock")
            .as_ref()
            .map(|account| account.id.clone())
            .unwrap_or_default();

        if content.is_empty() || peer_id.is_empty() {
            return;
        }

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
}

#[async_trait::async_trait]
impl ChannelProtocol for XianyuChannel {
    fn kind(&self) -> crate::channels::protocol::ChannelKind {
        crate::channels::protocol::ChannelKind::Xianyu
    }

    fn connection_state(&self) -> ConnectionState {
        *self.inner.state.read().expect("state lock")
    }

    fn set_inbound_listener(&self, listener: Arc<dyn InboundListener>) {
        *self.inner.listener.write().expect("listener lock") = Some(listener);
    }

    async fn connect(&self, account: &ChannelAccount) -> Result<(), ChannelError> {
        if *self.inner.state.read().expect("state lock") == ConnectionState::Connected {
            return Ok(());
        }
        *self.inner.account.write().expect("account lock") = Some(account.clone());

        self.stop_flag.store(false, Ordering::SeqCst);
        let (tx, mut rx) = mpsc::channel::<String>(64);
        *self.inner.writer.lock().await = Some(tx);

        // 终止旧的连接任务，避免重复连接循环。
        if let Some(old) = self.task.lock().await.take() {
            old.abort();
        }

        let this = self.clone();
        let stop_flag = self.stop_flag.clone();
        let task = tokio::spawn(async move {
            loop {
                if stop_flag.load(Ordering::SeqCst) {
                    break;
                }
                let result = this.run(&mut rx).await;
                if stop_flag.load(Ordering::SeqCst) {
                    break;
                }
                if let Err(error) = result {
                    this.inner
                        .set_state(ConnectionState::Error, Some(error.to_string()));
                    // 认证类错误（会话过期 / cookie 失效）重试无意义，停止自动重连，等待重新登录。
                    if is_auth_error(&error) {
                        break;
                    }
                } else {
                    this.inner.set_state(ConnectionState::Disconnected, None);
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        *self.task.lock().await = Some(task);
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), ChannelError> {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(task) = self.task.lock().await.take() {
            task.abort();
        }
        self.inner.set_state(ConnectionState::Disconnected, None);
        Ok(())
    }

    async fn send(&self, peer_id: &str, text: &str) -> Result<String, ChannelError> {
        let account = self
            .inner
            .account
            .read()
            .expect("account lock")
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
