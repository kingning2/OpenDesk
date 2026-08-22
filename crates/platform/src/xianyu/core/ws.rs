//! 闲鱼 WebSocket 长连接 — 收发消息核心。
//!
//! 协议流程（参考 XianyuAutoAgent 逆向）：
//! 1. `connect` 用 Cookie 握手建立 WS；
//! 2. 发 `/reg` 注册（携带 token）；
//! 3. 发 sync ack；
//! 4. 收帧循环：心跳 → 下行 ACK → `syncPushPackage` 解析会话/消息 → 上抛；
//! 5. 心跳 15s；token 每 1h 刷新并重连。

use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::HeaderValue;
use tokio_tungstenite::tungstenite::Message;

use super::api::XianyuApi;
use super::codec;
use crate::protocol::{
    ChannelAccount, ChannelError, ChannelInboundMessage, ChannelProtocol, ConnectionState,
    ConversationSync, HistoryMessage, InboundListener,
};
use crate::xianyu::message::{self, history};

use common::constants::xianyu;
use common::{DingDaError, DingDaResult};
use serde_json::Value;
use std::collections::HashMap;

const WS_URL: &str = xianyu::WS_URL;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);
const TOKEN_REFRESH_INTERVAL: Duration = Duration::from_secs(3600);

/// 把 Cookie 字符串收成合法 HTTP header 值（去掉 CR/LF 与非可见 ASCII）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 参数
/// - `cookie_str` — `name=value; ...` 原文
///
/// # 返回值
/// 可用于握手的 `HeaderValue`；非法时返回错误文案。
fn cookie_header_value(cookie_str: &str) -> Result<HeaderValue, String> {
    let cleaned: String = cookie_str
        .chars()
        .filter(|ch| *ch == '\t' || (' '..='~').contains(ch))
        .collect();
    HeaderValue::from_str(&cleaned).map_err(|error| format!("Cookie header 非法: {error}"))
}

/// 消息历史拉取参数（参考 goofish-cli `core/ws.py` `list_user_messages`）。
const HISTORY_PAGE_LIMIT: u32 = 50;
const HISTORY_MAX_PAGES: u32 = 3;
const HISTORY_RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);
/// `listUserMessages` 首次翻页游标（服务端约定的大数）。
const HISTORY_FIRST_CURSOR: i64 = 9_007_199_254_740_991;

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
        "登录态已过期",
        "请重新扫码登录",
        "Cookie 无效",
    ]
    .iter()
    .any(|keyword| text.contains(keyword))
}

/// 认证失效时给前端 / 日志的统一文案。
fn auth_error_user_message(error: &ChannelError) -> String {
    let detail = error.to_string();
    if detail.contains("登录态已过期") || detail.contains("请重新扫码登录") {
        detail
    } else {
        format!("登录态已过期，请重新扫码登录（{detail}）")
    }
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
    /// 请求-响应关联：mid → 响应 body 通道（`fetch_user_messages` 用）。
    pending: std::sync::Mutex<HashMap<String, mpsc::UnboundedSender<Value>>>,
    /// 等待 `/s/vulcan` 后才发出的请求帧文本。
    queued: std::sync::Mutex<Vec<String>>,
    /// 是否已收到 `/s/vulcan`（连接就绪，可直发请求）。
    vulcan_ready: std::sync::Mutex<bool>,
}

impl Inner {
    /// 锁中毒时恢复（`PoisonError::into_inner`），不 panic。
    fn read_state(&self) -> ConnectionState {
        *self
            .state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// 当前连接账号的 `(id, name)`，供会话列表日志标注归属。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-21
    ///
    /// # 返回值
    /// `(account_id, account_name)`；未绑定时两者为空串。
    fn account_label(&self) -> (String, String) {
        self.account
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map(|account| (account.id.clone(), account.name.clone()))
            .unwrap_or_default()
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

    /// 上抛会话列表同步（`userConvs`），应用层仅更新会话。
    fn notify_conversation(&self, sync: ConversationSync) {
        let listener = self
            .listener
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let Some(listener) = listener {
            tokio::spawn(async move {
                listener.on_conversation(sync).await;
            });
        }
    }

    /// 经出站通道向 WS 发送文本帧（若已连接）。
    async fn send_text(&self, frame: String) -> Result<(), String> {
        let writer = self.writer.lock().await;
        let Some(sender) = writer.as_ref() else {
            return Err("WS 未连接".to_string());
        };
        info!(frame = %frame, "WS 发送文本帧");
        sender.send(frame).await.map_err(|error| error.to_string())
    }
}

/// 闲鱼渠道协议实现。
#[derive(Clone)]
pub struct XianyuChannel {
    inner: Arc<Inner>,
    stop_flag: Arc<AtomicBool>,
    task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// 开发态：经本地 Host 帧隧道收发（协议代码在本进程）。
    dev_tunnel: bool,
}

impl Default for XianyuChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl XianyuChannel {
    /// 生产 / 默认：本进程直连闲鱼 WSS。
    pub fn new() -> Self {
        Self::with_tunnel(false)
    }

    /// 开发态：经 `127.0.0.1:10050` Host 隧道；上游由 Host 持有。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-21
    pub fn new_dev_tunnel() -> Self {
        Self::with_tunnel(true)
    }

    fn with_tunnel(dev_tunnel: bool) -> Self {
        Self {
            inner: Arc::new(Inner {
                account: RwLock::new(None),
                state: RwLock::new(ConnectionState::Disconnected),
                listener: RwLock::new(None),
                writer: tokio::sync::Mutex::new(None),
                pending: std::sync::Mutex::new(HashMap::new()),
                queued: std::sync::Mutex::new(Vec::new()),
                vulcan_ready: std::sync::Mutex::new(false),
            }),
            stop_flag: Arc::new(AtomicBool::new(false)),
            task: Arc::new(tokio::sync::Mutex::new(None)),
            dev_tunnel,
        }
    }

    /// 连接主循环：握手 → 注册 → ack → 收发帧。
    /// `outbound` 为跨重连的发送队列接收端。
    async fn run(&self, outbound: &mut mpsc::Receiver<String>) -> Result<(), ChannelError> {
        if self.dev_tunnel {
            return self.run_via_tunnel(outbound).await;
        }
        self.run_direct(outbound).await
    }

    /// 开发态隧道：Open Host → 收发 Text 帧（跳过本进程 token/`/reg`）。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-21
    async fn run_via_tunnel(
        &self,
        outbound: &mut mpsc::Receiver<String>,
    ) -> Result<(), ChannelError> {
        use super::dev_tunnel::{read_msg, write_msg, TunnelMsg, DEV_TUNNEL_ADDR};
        use tokio::net::TcpStream;

        let account = self
            .inner
            .account
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
            .ok_or_else(|| ChannelError::NotConnected("no account".into()))?;

        info!(
            account = %account.id,
            addr = DEV_TUNNEL_ADDR,
            "经开发态 Host 隧道附着（协议所有权在本进程）"
        );
        let stream = TcpStream::connect(DEV_TUNNEL_ADDR).await.map_err(|error| {
            ChannelError::Transport(format!(
                "连接 Channel Host 失败（{DEV_TUNNEL_ADDR}）: {error}"
            ))
        })?;
        let (mut reader, mut writer) = stream.into_split();
        write_msg(
            &mut writer,
            &TunnelMsg::Open {
                account_id: account.id.clone(),
                credential: account.credential.clone(),
            },
        )
        .await
        .map_err(|error| ChannelError::Transport(error.to_string()))?;

        match read_msg(&mut reader)
            .await
            .map_err(|error| ChannelError::Transport(error.to_string()))?
        {
            Some(TunnelMsg::Ready { .. }) => {}
            Some(TunnelMsg::Error { message }) => {
                // Host 已带上 Session 过期等原文；按内容归类为认证/协议错误，避免 30s 误重连。
                if is_auth_error(&ChannelError::Protocol(message.clone())) {
                    return Err(ChannelError::Protocol(message));
                }
                return Err(ChannelError::Transport(message));
            }
            Some(other) => {
                return Err(ChannelError::Transport(format!("Host 首包异常: {other:?}")));
            }
            None => {
                return Err(ChannelError::Transport("Host 关闭连接".into()));
            }
        }

        self.inner
            .set_state(ConnectionState::Connected, Some("隧道已附着".into()));
        info!(account = %account.id, "已取得上游帧所有权，开始本进程协议循环");

        {
            *self
                .inner
                .vulcan_ready
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = false;
            self.inner
                .queued
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clear();
        }

        let cookie_str = super::cookies::cookies_to_string(&super::cookies::parse_credential(
            &account.credential,
        ));
        {
            let this = self.clone();
            let cookie = cookie_str;
            let account_id = account.id.clone();
            tokio::spawn(async move {
                this.sync_sessions(&cookie, &account_id).await;
            });
        }

        let mut last_heartbeat = Instant::now();
        let vulcan_deadline = Instant::now() + Duration::from_secs(8);

        while !self.stop_flag.load(Ordering::SeqCst) {
            if Instant::now() >= vulcan_deadline {
                let ready = *self
                    .inner
                    .vulcan_ready
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if !ready {
                    warn!(
                        account = %account.id,
                        "隧道模式等待 /s/vulcan 超时，降级直发"
                    );
                    self.flush_queued().await;
                }
            }
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(500)) => {
                    if last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
                        let frame = message::heartbeat_frame().to_string();
                        if write_msg(&mut writer, &TunnelMsg::Text { data: frame })
                            .await
                            .is_err()
                        {
                            return Ok(());
                        }
                        last_heartbeat = Instant::now();
                    }
                }
                Some(frame) = outbound.recv() => {
                    if write_msg(&mut writer, &TunnelMsg::Text { data: frame })
                        .await
                        .is_err()
                    {
                        return Ok(());
                    }
                }
                incoming = read_msg(&mut reader) => {
                    match incoming {
                        Ok(Some(TunnelMsg::Text { data })) => {
                            self.handle_text_frame(&data).await;
                        }
                        Ok(Some(TunnelMsg::Error { message })) => {
                            return Err(ChannelError::Transport(message));
                        }
                        Ok(Some(_)) => {}
                        Ok(None) => return Ok(()),
                        Err(error) => {
                            return Err(ChannelError::Transport(error.to_string()));
                        }
                    }
                }
            }
        }
        let _ = write_msg(&mut writer, &TunnelMsg::Detach).await;
        Ok(())
    }

    /// 本进程直连闲鱼 WSS（生产路径）。
    async fn run_direct(&self, outbound: &mut mpsc::Receiver<String>) -> Result<(), ChannelError> {
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

        // 必须先用 URL 生成带 Sec-WebSocket-Key / Upgrade 的握手请求，再附加 Cookie。
        // 手写 Request::builder 不会自动补这些头，tungstenite 会报
        // Missing, duplicated or incorrect header sec-websocket-key。
        let mut request = WS_URL
            .into_client_request()
            .map_err(|error| ChannelError::Protocol(format!("构造 WS 请求失败: {error}")))?;
        {
            let headers = request.headers_mut();
            headers.insert(
                "Cookie",
                cookie_header_value(&cookie_str).map_err(|error| ChannelError::Protocol(error))?,
            );
            headers.insert("Origin", HeaderValue::from_static(xianyu::WEB_ORIGIN));
            headers.insert(
                "User-Agent",
                HeaderValue::from_static(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                     (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
                ),
            );
        }
        let (mut sink, mut stream) = connect_async(request)
            .await
            .map_err(|error| ChannelError::Transport(format!("ws 连接失败: {error}")))?
            .0
            .split();

        self.inner
            .set_state(ConnectionState::Connected, Some("连接成功".into()));
        info!(account = %account.id, url = WS_URL, "WebSocket 连接已建立");

        let reg = message::register_frame(&device_id, &token);
        info!(account = %account.id, frame = %reg.to_string(), "WS 发送注册帧");
        sink.send(Message::Text(reg.to_string()))
            .await
            .map_err(|error| ChannelError::Transport(error.to_string()))?;
        let ack = message::sync_ack_frame();
        info!(account = %account.id, frame = %ack.to_string(), "WS 发送 ackDiff 帧");
        sink.send(Message::Text(ack.to_string()))
            .await
            .map_err(|error| ChannelError::Transport(error.to_string()))?;
        info!(account = %account.id, "设备注册帧与同步确认已发送，开始监听消息");

        // 每次重连重置就绪态，避免沿用上一连接的 vulcan_ready。
        {
            *self
                .inner
                .vulcan_ready
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = false;
            self.inner
                .queued
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clear();
        }

        // 会话 HTTP 同步放到后台，切勿堵在读循环前，否则 /s/vulcan 积压未读，
        // 消息历史会一直「等待 vulcan」直到超时（请求其实从未发出）。
        {
            let this = self.clone();
            let cookie = cookie_str.clone();
            let account_id = account.id.clone();
            tokio::spawn(async move {
                this.sync_sessions(&cookie, &account_id).await;
            });
        }

        let mut last_heartbeat = Instant::now();
        let last_token_refresh = Instant::now();
        let vulcan_deadline = Instant::now() + Duration::from_secs(8);

        while !self.stop_flag.load(Ordering::SeqCst) {
            // 超时仍未收到 /s/vulcan：降级直发，避免历史/业务请求永久挂起。
            if Instant::now() >= vulcan_deadline {
                let ready = *self
                    .inner
                    .vulcan_ready
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if !ready {
                    warn!(
                        account = %account.id,
                        "等待 /s/vulcan 超时，降级为直接发送排队请求"
                    );
                    self.flush_queued().await;
                }
            }
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(500)) => {
                    if last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
                        let frame = message::heartbeat_frame();
                        info!(account = %account.id, frame = %frame.to_string(), "WS 发送心跳帧");
                        if sink.send(Message::Text(frame.to_string())).await.is_err() {
                            return Ok(());
                        }
                        last_heartbeat = Instant::now();
                    }
                    if last_token_refresh.elapsed() >= TOKEN_REFRESH_INTERVAL {
                        info!(account = %account.id, "token 到期，触发重连刷新");
                        return Ok(()); // 触发重连以刷新 token
                    }
                }
                Some(frame) = outbound.recv() => {
                    info!(account = %account.id, frame = %frame, "WS 发送文本帧");
                    if sink.send(Message::Text(frame)).await.is_err() {
                        return Ok(());
                    }
                }
                incoming = stream.next() => {
                    match incoming {
                        Some(Ok(Message::Text(text))) => {
                            info!(account = %account.id, frame = %text, "WS 收到文本帧");
                            self.handle_text_frame(&text).await;
                        }
                        Some(Ok(Message::Binary(bin))) => {
                            info!(account = %account.id, len = bin.len(), "WS 收到二进制帧");
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

    async fn handle_text_frame(&self, text: &str) {
        let Ok(msg) = serde_json::from_str::<Value>(text) else {
            return;
        };
        let lwp = msg.get("lwp").and_then(Value::as_str).unwrap_or("");

        // 对齐 goofish-cli：下行业务帧回 ACK，避免服务端停推。
        if let Some(headers) = msg.get("headers") {
            if headers
                .get("mid")
                .and_then(Value::as_str)
                .is_some_and(|m| !m.is_empty())
            {
                let ack = message::ack_frame(headers).to_string();
                if let Err(error) = self.inner.send_text(ack).await {
                    warn!(%error, "发送下行 ACK 失败");
                }
            }
        }

        // 连接就绪推送：此刻起可直发 LWP 请求（推包在后续独立帧到达）。
        if lwp == "/s/vulcan" {
            info!("收到 /s/vulcan，flush 排队请求帧");
            self.flush_queued().await;
            return;
        }

        let (account_id, account_name) = self.inner.account_label();

        if let Some(convs) = msg.pointer("/body/userConvs").and_then(Value::as_array) {
            let synced = self.sync_user_convs(convs, &account_id).await;
            info!(
                account_id = %account_id,
                account_name = %account_name,
                count = synced,
                source = "userConvs",
                "【会话列表】账号「{account_name}」({account_id}) 来源=userConvs 获取 {synced} 条"
            );
            if synced > 0 {
                self.inner.set_state(
                    ConnectionState::Connected,
                    Some("conversations_synced".into()),
                );
            }
            return;
        }

        // 全量同步推包（对齐 goofish-cli collect_session_cids）。
        if msg.pointer("/body/syncPushPackage").is_some() {
            let batch = message::parse_sync_push_package(&msg);
            let mut synced = 0usize;
            for session in &batch.sessions {
                // peer 未知时用 cid 占位；真消息到达后按 cid 合并。
                let peer_id = session.cid.clone();
                let updated_at = if session.updated_at.is_empty() {
                    message::now_ms().to_string()
                } else {
                    session.updated_at.clone()
                };
                let sync = ConversationSync {
                    account_id: account_id.clone(),
                    cid: session.cid.clone(),
                    peer_id,
                    item_id: session.item_id.clone(),
                    item_title: session.item_title.clone(),
                    updated_at,
                };
                self.inner.notify_conversation(sync);
                synced += 1;
            }
            for pushed in &batch.messages {
                let inbound = ChannelInboundMessage {
                    account_id: account_id.clone(),
                    peer_id: pushed.peer_id.clone(),
                    peer_name: pushed.peer_name.clone(),
                    item_id: pushed.item_id.clone(),
                    cid: pushed.cid.clone(),
                    content: pushed.content.clone(),
                    created_at_ms: pushed.created_at_ms,
                };
                self.inner.notify_message(inbound);
            }
            info!(
                account_id = %account_id,
                account_name = %account_name,
                count = synced,
                messages = batch.messages.len(),
                source = "syncPushPackage",
                "【会话列表】账号「{account_name}」({account_id}) 来源=syncPushPackage 获取 {synced} 条会话、{} 条消息",
                batch.messages.len()
            );
            if synced > 0 {
                self.inner.set_state(
                    ConnectionState::Connected,
                    Some("conversations_synced".into()),
                );
            }
            return;
        }

        // 请求-响应关联：命中 mid 则把 body 交给等待方。
        let mid = msg
            .pointer("/headers/mid")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if mid.is_empty() {
            return;
        }
        let responder = {
            let mut pending = self
                .inner
                .pending
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            pending.remove(&mid)
        };
        if let Some(responder) = responder {
            let body = msg.get("body").cloned().unwrap_or(Value::Null);
            let _ = responder.send(body);
        } else {
            let pending = self
                .inner
                .pending
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .len();
            if pending > 0 {
                warn!(lwp = %lwp, mid = %mid, pending, "收到文本帧但无匹配 mid");
            }
        }
    }

    /// 标记连接就绪并把排队中的 LWP 请求帧发出。
    async fn flush_queued(&self) {
        {
            let mut ready = self
                .inner
                .vulcan_ready
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *ready = true;
        }
        let frames: Vec<String> = {
            let mut queued = self
                .inner
                .queued
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            std::mem::take(&mut *queued)
        };
        for frame in frames {
            if let Err(error) = self.inner.send_text(frame).await {
                warn!(%error, "发送排队请求帧失败");
            }
        }
    }

    /// 超时或取消时清掉 pending mid，避免泄漏。
    fn clear_pending(&self, mid: &str) {
        let mut pending = self
            .inner
            .pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        pending.remove(mid);
    }

    /// 拉取会话完整消息历史（`MessageManager/listUserMessages`，参考 goofish-cli）。
    pub async fn fetch_user_messages(
        &self,
        cid: &str,
        limit: u32,
    ) -> DingDaResult<Vec<HistoryMessage>> {
        let mut all = Vec::new();
        let mut cursor = HISTORY_FIRST_CURSOR;
        for _ in 0..HISTORY_MAX_PAGES {
            let frame = message::list_user_messages_frame(cid, cursor, limit);
            let mid = frame
                .pointer("/headers/mid")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let (tx, mut rx) = mpsc::unbounded_channel::<Value>();
            {
                let mut pending = self
                    .inner
                    .pending
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                pending.insert(mid.clone(), tx);
            }
            let already_ready = *self
                .inner
                .vulcan_ready
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            {
                let mut queued = self
                    .inner
                    .queued
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                queued.push(frame.to_string());
            }
            if already_ready {
                info!(cid = %cid, mid = %mid, "消息历史请求发送（vulcan 已就绪）");
                self.flush_queued().await;
            } else {
                info!(cid = %cid, mid = %mid, "消息历史请求入队，等待 vulcan 就绪");
            }

            let body = match tokio::time::timeout(HISTORY_RESPONSE_TIMEOUT, rx.recv()).await {
                Ok(Some(body)) => body,
                Ok(None) => {
                    self.clear_pending(&mid);
                    return Err(DingDaError::channel("消息历史响应通道已关闭"));
                }
                Err(_) => {
                    self.clear_pending(&mid);
                    // 超时兜底：若仍未就绪则强制 flush 一次，便于下次更快。
                    let ready = *self
                        .inner
                        .vulcan_ready
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    if !ready {
                        warn!(cid = %cid, mid = %mid, "历史超时且 vulcan 未就绪，强制 flush");
                        self.flush_queued().await;
                    }
                    return Err(DingDaError::channel("拉取消息历史超时"));
                }
            };

            if let Some(code) = body.get("code").and_then(Value::as_i64) {
                if code != 200 {
                    return Err(DingDaError::channel(format!(
                        "listUserMessages 返回 code={code}"
                    )));
                }
            }

            let models = body
                .get("userMessageModels")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for model in models {
                if let Some(msg) = history::parse_history_message(&model) {
                    all.push(msg);
                }
            }

            let has_more = body.get("hasMore").and_then(Value::as_i64).unwrap_or(0) == 1;
            if !has_more {
                break;
            }
            match body.get("nextCursor").and_then(Value::as_i64) {
                Some(next) if next > 0 => cursor = next,
                _ => break,
            }
        }
        Ok(all)
    }

    async fn handle_binary_frame(&self, bin: &[u8]) {
        let parsed: Result<rmpv::Value, _> = rmpv::decode::read_value(&mut &bin[..]);
        let Ok(frame) = parsed else {
            return;
        };
        info!(frame = %super::mtop::truncate_log(&format!("{frame:?}"), 2500), "WS 二进制帧内容");
        if !codec::is_chat_message(&frame) {
            debug!("收到非聊天二进制帧");
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
        // cid 为 `["1"]["2"]` 的 `cid@goofish`，取裸数字部分作为会话 id。
        let cid = codec::get_string(&frame, codec::MSG_TYPE)
            .map(|raw| message::extract_cid(&raw))
            .unwrap_or_default();
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
            cid,
            content,
            created_at_ms,
        };
        self.inner.notify_message(inbound);
    }

    /// 连接成功后拉取 mtop 会话列表，把每条会话的最后一条消息同步进来
    /// （WebSocket 推送前的待回复消息补全）。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-20
    async fn sync_sessions(&self, cookie_str: &str, account_id: &str) {
        let account_name = self
            .inner
            .account
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map(|account| account.name.clone())
            .unwrap_or_default();
        // fetchNum 调大：session.sync 一次最多返回 fetchNum 条，默认 50 会漏掉后面的活跃会话。
        match super::session::fetch_sessions(cookie_str, 500).await {
            Ok((sessions, _)) => {
                if sessions.is_empty() {
                    info!(
                        account_id = %account_id,
                        account_name = %account_name,
                        count = 0u32,
                        source = "session.sync",
                        "【会话列表】账号「{account_name}」({account_id}) 来源=session.sync 获取 0 条"
                    );
                    return;
                }
                info!(
                    account_id = %account_id,
                    account_name = %account_name,
                    count = sessions.len(),
                    source = "session.sync",
                    "【会话列表】账号「{account_name}」({account_id}) 来源=session.sync 正在入库 {} 条",
                    sessions.len()
                );
                let total = sessions.len();
                let mut synced = 0u32;
                for session in sessions {
                    let peer_id = if !session.peer_id.is_empty() {
                        session.peer_id
                    } else {
                        message::extract_cid(&session.session_id)
                    };
                    if peer_id.is_empty() {
                        continue;
                    }
                    synced += 1;

                    let content = session.last_msg;

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
                        cid: message::extract_cid(&session.session_id),
                        content,
                        created_at_ms,
                    };
                    self.inner.notify_message(inbound);
                }
                info!(
                    account_id = %account_id,
                    account_name = %account_name,
                    count = synced,
                    total,
                    source = "session.sync",
                    "【会话列表】账号「{account_name}」({account_id}) 来源=session.sync 完成 {synced}/{total} 条"
                );
            }
            Err(error) => {
                let detail = error.to_string();
                warn!(
                    account_id = %account_id,
                    account_name = %account_name,
                    source = "session.sync",
                    %detail,
                    "【会话列表】账号「{account_name}」({account_id}) 来源=session.sync 拉取失败"
                );
                if super::risk::is_risk_control_text(&detail) {
                    self.inner.set_state(ConnectionState::Error, Some(detail));
                }
            }
        }
    }

    /// 解析 `body.userConvs`（WS 下发的完整会话列表）并入库。
    ///
    /// 字段参考：`singleChatUserConversation.singleChatConversation`（cid/extension）
    /// + `singleChatUserConversation.modifyTime`（更新时间）。
    async fn sync_user_convs(&self, convs: &[Value], account_id: &str) -> usize {
        let mut synced = 0usize;
        for conv in convs {
            let visible = conv
                .pointer("/singleChatUserConversation/visible")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            if visible == 0 {
                continue;
            }
            let Some(sc) = conv.pointer("/singleChatUserConversation/singleChatConversation")
            else {
                continue;
            };
            let Some(cid_raw) = sc.get("cid").and_then(Value::as_str) else {
                continue;
            };
            let cid = message::extract_cid(cid_raw);
            if cid.is_empty() {
                continue;
            }
            let extension = sc.get("extension").unwrap_or(&Value::Null);
            // extUserId 在部分推送里是卖家自身；缺失或不合理时用 cid 占位，后续按 cid 合并。
            let peer_id = extension
                .get("extUserId")
                .and_then(Value::as_str)
                .filter(|id| !id.is_empty())
                .unwrap_or(cid.as_str())
                .to_string();
            let item_id = extension
                .get("itemId")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let item_title = extension
                .get("itemTitle")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let updated_at = conv
                .pointer("/singleChatUserConversation/modifyTime")
                .and_then(Value::as_i64)
                .unwrap_or(0)
                .to_string();
            let sync = ConversationSync {
                account_id: account_id.to_string(),
                cid,
                peer_id,
                item_id,
                item_title,
                updated_at,
            };
            self.inner.notify_conversation(sync);
            synced += 1;
        }
        synced
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

    async fn fetch_history(&self, cid: &str) -> DingDaResult<Vec<HistoryMessage>> {
        self.fetch_user_messages(cid, HISTORY_PAGE_LIMIT).await
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
        let flags = common::constants::FeatureFlags::from_env();
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
                        let message = auth_error_user_message(&error);
                        error!(
                            account = %account_id_for_log,
                            %error,
                            "闲鱼认证失效（会话过期 / cookie 无效），停止自动重连，请重新登录"
                        );
                        this.inner.set_state(ConnectionState::Error, Some(message));
                        break;
                    }
                    // 风控拦截：非会话过期，指数退避后自动重试。
                    if is_risk_control(&error) {
                        if !flags.channel_auto_reconnect {
                            warn!(
                                account = %account_id_for_log,
                                %error,
                                "闲鱼风控拦截，且已关闭自动重连（DINGDA_CHANNEL_AUTO_RECONNECT=0）"
                            );
                            this.inner
                                .set_state(ConnectionState::Error, Some(error.to_string()));
                            break;
                        }
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
                    warn!(
                        account = %account_id_for_log,
                        %error,
                        retry_in_secs = flags.channel_reconnect_delay_secs,
                        "闲鱼连接异常，等待后重连"
                    );
                    this.inner
                        .set_state(ConnectionState::Error, Some(error.to_string()));
                } else {
                    consecutive_failures = 0;
                    info!(
                        account = %account_id_for_log,
                        retry_in_secs = flags.channel_reconnect_delay_secs,
                        "闲鱼连接已断开，等待后重连"
                    );
                    this.inner.set_state(ConnectionState::Disconnected, None);
                }
                if !flags.channel_auto_reconnect {
                    warn!(
                        account = %account_id_for_log,
                        "已关闭自动重连（DINGDA_CHANNEL_AUTO_RECONNECT=0），请手动连接"
                    );
                    break;
                }
                tokio::time::sleep(Duration::from_secs(flags.channel_reconnect_delay_secs)).await;
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

    async fn send(&self, cid: &str, peer_id: &str, text: &str) -> DingDaResult<String> {
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
        let frame = message::send_message_frame(cid, peer_id, &my_id, text);

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
        assert!(is_auth_error(&proto(
            "登录态已过期或 Cookie 无效，请重新扫码登录（FAIL_SYS_SESSION_EXPIRED）"
        )));
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
