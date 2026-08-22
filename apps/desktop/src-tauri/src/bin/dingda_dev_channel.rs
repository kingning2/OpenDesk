//! 开发态 Channel Host — 持有上游闲鱼 WSS，经本地 TCP 帧隧道移交协议所有权。
//!
//! - 空闲：Host 发心跳，不断上游
//! - Tauri `Open`/`Attach`：双向转发文本帧，协议逻辑跑在 Tauri
//! - Tauri 退出：隧道断 → 所有权归还 Host
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-21

#![cfg_attr(not(platform_xianyu), allow(dead_code, unused_imports))]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use platform::xianyu::api::XianyuApi;
use platform::xianyu::cookies::{self, cookies_to_string, parse_credential};
use platform::xianyu::dev_tunnel::{
    read_msg, write_msg, TunnelMsg, TunnelSession, DEV_TUNNEL_ADDR,
};
use platform::xianyu::message::{heartbeat_frame, register_frame, sync_ack_frame};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::HeaderValue;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);

/// 会话控制命令。
enum SessionCmd {
    /// Tauri 附着：上行/下行文本通道。
    Attach {
        to_tauri: mpsc::Sender<String>,
        from_tauri: mpsc::Receiver<String>,
        done: oneshot::Sender<Result<(), String>>,
    },
    /// 查询是否在跑。
    Ping { done: oneshot::Sender<bool> },
}

/// 全局会话表项。
struct SessionEntry {
    cmd: mpsc::Sender<SessionCmd>,
    attached: Arc<Mutex<bool>>,
}

type SessionMap = Arc<Mutex<HashMap<String, SessionEntry>>>;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("创建 tokio runtime 失败");

    if runtime.block_on(port_in_use()) {
        info!(addr = DEV_TUNNEL_ADDR, "Channel Host 已在运行，本进程退出");
        return;
    }

    let pid = std::process::id();
    if let Err(error) = write_pid_file(pid) {
        warn!(%error, "写入 PID 文件失败（继续启动）");
    }

    let sessions: SessionMap = Arc::new(Mutex::new(HashMap::new()));
    runtime.block_on(async move {
        let listener = TcpListener::bind(DEV_TUNNEL_ADDR)
            .await
            .unwrap_or_else(|error| panic!("绑定 {DEV_TUNNEL_ADDR} 失败: {error}"));
        info!(
            addr = DEV_TUNNEL_ADDR,
            pid, "开发态 Channel Host（帧隧道）启动"
        );
        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let sessions = sessions.clone();
                    tokio::spawn(async move {
                        if let Err(error) = handle_client(stream, sessions).await {
                            warn!(%peer, %error, "隧道连接结束");
                        }
                    });
                }
                Err(error) => warn!(%error, "accept 失败"),
            }
        }
    });
}

async fn port_in_use() -> bool {
    TcpStream::connect(DEV_TUNNEL_ADDR).await.is_ok()
}

fn pid_file_path() -> PathBuf {
    if let Ok(path) = std::env::var("DINGDA_DEV_CHANNEL_PID_FILE") {
        return PathBuf::from(path);
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("dingda")
        .join("dev-channel.pid")
}

fn write_pid_file(pid: u32) -> std::io::Result<()> {
    let path = pid_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, pid.to_string())?;
    info!(path = %path.display(), pid, "已写入 PID 文件");
    Ok(())
}

async fn handle_client(stream: TcpStream, sessions: SessionMap) -> Result<(), String> {
    let (mut reader, mut writer) = stream.into_split();
    let first = read_msg(&mut reader)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "对端关闭".to_string())?;

    match first {
        TunnelMsg::List => {
            let list = list_sessions(&sessions).await;
            write_msg(&mut writer, &TunnelMsg::ListOk { sessions: list })
                .await
                .map_err(|error| error.to_string())?;
            Ok(())
        }
        TunnelMsg::Open {
            account_id,
            credential,
        } => {
            // 启动失败必须回传 Error，否则 Tauri 只看到 TCP 关闭，误当成「Host 关闭连接」去重连。
            if let Err(error) = ensure_session(&sessions, &account_id, &credential).await {
                let message = format_session_boot_error(&error);
                warn!(account = %account_id, %message, "Open 上游会话失败");
                write_msg(
                    &mut writer,
                    &TunnelMsg::Error {
                        message: message.clone(),
                    },
                )
                .await
                .ok();
                return Err(message);
            }
            if let Err(error) = attach_and_relay(&sessions, &account_id, reader, writer).await {
                let message = format_session_boot_error(&error);
                warn!(account = %account_id, %message, "Attach 转发失败");
                return Err(message);
            }
            Ok(())
        }
        TunnelMsg::Attach { account_id } => {
            if !session_alive(&sessions, &account_id).await {
                write_msg(
                    &mut writer,
                    &TunnelMsg::Error {
                        message: format!("无上游会话，请先 Open: {account_id}"),
                    },
                )
                .await
                .map_err(|error| error.to_string())?;
                return Err("no session".into());
            }
            attach_and_relay(&sessions, &account_id, reader, writer).await
        }
        other => {
            write_msg(
                &mut writer,
                &TunnelMsg::Error {
                    message: format!("首包必须是 open/attach/list，收到: {other:?}"),
                },
            )
            .await
            .ok();
            Err("bad first message".into())
        }
    }
}

async fn list_sessions(sessions: &SessionMap) -> Vec<TunnelSession> {
    let guard = sessions.lock().await;
    let mut out = Vec::new();
    for (account_id, entry) in guard.iter() {
        let attached = *entry.attached.lock().await;
        out.push(TunnelSession {
            account_id: account_id.clone(),
            state: if attached {
                "attached".into()
            } else {
                "idle".into()
            },
        });
    }
    out
}

async fn session_alive(sessions: &SessionMap, account_id: &str) -> bool {
    let cmd = {
        let guard = sessions.lock().await;
        guard.get(account_id).map(|entry| entry.cmd.clone())
    };
    let Some(cmd) = cmd else {
        return false;
    };
    let (tx, rx) = oneshot::channel();
    if cmd.send(SessionCmd::Ping { done: tx }).await.is_err() {
        return false;
    }
    rx.await.unwrap_or(false)
}

/// 把上游启动错误整理成客户端可读文案（Session 过期时点明重新登录）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
fn format_session_boot_error(error: &str) -> String {
    if error.contains("FAIL_SYS_SESSION_EXPIRED")
        || error.contains("Session过期")
        || error.contains("SESSION_EXPIRED")
        || error.contains("cookie 缺少")
    {
        format!("登录态已过期或 Cookie 无效，请重新扫码登录（{error}）")
    } else {
        error.to_string()
    }
}

async fn ensure_session(
    sessions: &SessionMap,
    account_id: &str,
    credential: &str,
) -> Result<(), String> {
    if session_alive(sessions, account_id).await {
        info!(account = %account_id, "复用已有上游 WSS");
        return Ok(());
    }
    info!(account = %account_id, "建立上游闲鱼 WSS");
    let (cmd_tx, cmd_rx) = mpsc::channel(8);
    let attached = Arc::new(Mutex::new(false));
    let (boot_tx, boot_rx) = oneshot::channel::<Result<(), String>>();
    {
        let mut guard = sessions.lock().await;
        // 清理死会话
        guard.retain(|_, entry| !entry.cmd.is_closed());
        guard.insert(
            account_id.to_string(),
            SessionEntry {
                cmd: cmd_tx,
                attached: attached.clone(),
            },
        );
    }
    let account_id = account_id.to_string();
    let credential = credential.to_string();
    let account_for_wait = account_id.clone();
    tokio::spawn(async move {
        let result = session_actor(account_id.clone(), credential, cmd_rx, attached, boot_tx).await;
        if let Err(error) = result {
            warn!(account = %account_id, %error, "上游会话退出");
        }
    });

    // 优先接收 actor 启动结果（token / cookie 失败会立刻带回原文）。
    match tokio::time::timeout(Duration::from_secs(5), boot_rx).await {
        Ok(Ok(Ok(()))) => {
            for _ in 0..20 {
                if session_alive(sessions, &account_for_wait).await {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            sessions.lock().await.remove(&account_for_wait);
            Err(format!("上游会话启动超时: {account_for_wait}"))
        }
        Ok(Ok(Err(error))) => {
            sessions.lock().await.remove(&account_for_wait);
            Err(error)
        }
        Ok(Err(_)) => {
            // boot 通道提前关闭：再等 Ping 兜底。
            for _ in 0..20 {
                if session_alive(sessions, &account_for_wait).await {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            sessions.lock().await.remove(&account_for_wait);
            Err(format!("上游会话启动超时: {account_for_wait}"))
        }
        Err(_) => {
            sessions.lock().await.remove(&account_for_wait);
            Err(format!("上游会话启动超时: {account_for_wait}"))
        }
    }
}

async fn attach_and_relay<R, W>(
    sessions: &SessionMap,
    account_id: &str,
    mut reader: R,
    mut writer: W,
) -> Result<(), String>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    /// Ready 之前失败必须回传 Error，否则 Tauri 只看到 TCP EOF →「Host 关闭连接」。
    async fn fail_before_ready<W>(writer: &mut W, message: String) -> Result<(), String>
    where
        W: AsyncWrite + Unpin,
    {
        let _ = write_msg(
            writer,
            &TunnelMsg::Error {
                message: message.clone(),
            },
        )
        .await;
        Err(message)
    }

    let (to_tauri_tx, mut to_tauri_rx) = mpsc::channel::<String>(64);
    let (from_tauri_tx, from_tauri_rx) = mpsc::channel::<String>(64);
    let cmd = {
        let guard = sessions.lock().await;
        match guard.get(account_id).map(|entry| entry.cmd.clone()) {
            Some(cmd) => cmd,
            None => {
                return fail_before_ready(&mut writer, "session missing".into()).await;
            }
        }
    };
    let (done_tx, done_rx) = oneshot::channel();
    if cmd
        .send(SessionCmd::Attach {
            to_tauri: to_tauri_tx,
            from_tauri: from_tauri_rx,
            done: done_tx,
        })
        .await
        .is_err()
    {
        return fail_before_ready(&mut writer, "session actor 已退出".into()).await;
    }
    match done_rx.await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => return fail_before_ready(&mut writer, error).await,
        Err(_) => return fail_before_ready(&mut writer, "attach ack 丢失".into()).await,
    }

    write_msg(
        &mut writer,
        &TunnelMsg::Ready {
            account_id: account_id.to_string(),
        },
    )
    .await
    .map_err(|error| error.to_string())?;
    info!(account = %account_id, "协议所有权已交给 Tauri（帧转发中）");

    let account_for_log = account_id.to_string();
    let pump_up = async {
        while let Some(msg) = read_msg(&mut reader).await.map_err(|e| e.to_string())? {
            match msg {
                TunnelMsg::Text { data } => {
                    if from_tauri_tx.send(data).await.is_err() {
                        break;
                    }
                }
                TunnelMsg::Detach => break,
                other => {
                    warn!(?other, "隧道忽略非 Text 帧");
                }
            }
        }
        Ok::<(), String>(())
    };
    let pump_down = async {
        while let Some(data) = to_tauri_rx.recv().await {
            if write_msg(&mut writer, &TunnelMsg::Text { data })
                .await
                .is_err()
            {
                break;
            }
        }
        Ok::<(), String>(())
    };

    tokio::select! {
        result = pump_up => { let _ = result; }
        result = pump_down => { let _ = result; }
    }

    info!(account = %account_for_log, "Tauri 隧道断开，所有权归还 Host");
    Ok(())
}

async fn session_actor(
    account_id: String,
    credential: String,
    mut cmd_rx: mpsc::Receiver<SessionCmd>,
    attached_flag: Arc<Mutex<bool>>,
    boot_tx: oneshot::Sender<Result<(), String>>,
) -> Result<(), String> {
    let boot_result = async {
        let cookie_str = cookies_to_string(&parse_credential(&credential));
        let api = XianyuApi::new(&cookie_str).map_err(|error| error.to_string())?;
        let cookies_map = parse_credential(&credential);
        let device_id = cookies::device_id(&cookies_map).ok_or("cookie 缺少 unb")?;
        let token = api.fetch_token().await.map_err(|error| error.to_string())?;

        let mut request = common::constants::xianyu::WS_URL
            .into_client_request()
            .map_err(|error| error.to_string())?;
        {
            let headers = request.headers_mut();
            let cleaned: String = cookie_str
                .chars()
                .filter(|ch| *ch == '\t' || (' '..='~').contains(ch))
                .collect();
            headers.insert(
                "Cookie",
                HeaderValue::from_str(&cleaned).map_err(|error| error.to_string())?,
            );
            headers.insert(
                "Origin",
                HeaderValue::from_static(common::constants::xianyu::WEB_ORIGIN),
            );
            headers.insert(
                "User-Agent",
                HeaderValue::from_static(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                     (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
                ),
            );
        }
        let (ws, _) = connect_async(request)
            .await
            .map_err(|error| format!("ws 连接失败: {error}"))?;
        let (mut sink, stream) = ws.split();

        let reg = register_frame(&device_id, &token);
        sink.send(Message::Text(reg.to_string()))
            .await
            .map_err(|error| error.to_string())?;
        sink.send(Message::Text(sync_ack_frame().to_string()))
            .await
            .map_err(|error| error.to_string())?;
        info!(account = %account_id, "上游 WSS 已注册");
        Ok::<_, String>((sink, stream))
    }
    .await;

    let (mut sink, mut stream) = match boot_result {
        Ok(pair) => {
            let _ = boot_tx.send(Ok(()));
            pair
        }
        Err(error) => {
            let _ = boot_tx.send(Err(error.clone()));
            return Err(error);
        }
    };

    let mut to_tauri: Option<mpsc::Sender<String>> = None;
    let mut from_tauri: Option<mpsc::Receiver<String>> = None;
    let mut last_heartbeat = Instant::now();

    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(SessionCmd::Attach { to_tauri: tx, from_tauri: rx, done }) => {
                        // 重连时旧隧道可能尚未拆完：覆盖附着，丢弃旧通道。
                        if to_tauri.is_some() {
                            warn!(
                                account = %account_id,
                                "检测到重复附着，覆盖旧 Tauri 隧道"
                            );
                        }
                        to_tauri = Some(tx);
                        from_tauri = Some(rx);
                        *attached_flag.lock().await = true;
                        let _ = done.send(Ok(()));
                        info!(account = %account_id, "上游进入转发模式");
                    }
                    Some(SessionCmd::Ping { done }) => {
                        let _ = done.send(true);
                    }
                    None => break,
                }
            }
            incoming = stream.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if let Some(tx) = &to_tauri {
                            if tx.send(text).await.is_err() {
                                to_tauri = None;
                                from_tauri = None;
                                *attached_flag.lock().await = false;
                            }
                        }
                        // idle：丢弃业务帧，保持读循环以免背压
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        let _ = sink.send(Message::Pong(payload)).await;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(error)) => return Err(error.to_string()),
                    None => return Err("上游关闭".into()),
                }
            }
            frame = async {
                match from_tauri.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending::<Option<String>>().await,
                }
            } => {
                match frame {
                    Some(data) => {
                        if sink.send(Message::Text(data)).await.is_err() {
                            return Err("上游写入失败".into());
                        }
                    }
                    None => {
                        // Tauri 写端关闭
                        to_tauri = None;
                        from_tauri = None;
                        *attached_flag.lock().await = false;
                        info!(account = %account_id, "Tauri 写端关闭，Host 收回所有权");
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(500)) => {
                if to_tauri.is_none() && last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
                    let frame = heartbeat_frame();
                    if sink.send(Message::Text(frame.to_string())).await.is_err() {
                        return Err("心跳失败".into());
                    }
                    last_heartbeat = Instant::now();
                }
            }
        }
    }
    Ok(())
}
