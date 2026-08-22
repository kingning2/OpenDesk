//! 开发态 Channel Host 生命周期 — 拉起常驻进程，不持有业务协议。
//!
//! 协议经 [`platform::xianyu::XianyuChannel::new_dev_tunnel`] 走帧隧道；
//! 本模块只负责：确保 Host 进程存活、列出可复用会话、启动后自动 connect。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-21

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use platform::protocol::dispatcher::ChannelDispatcher;
use platform::protocol::ChannelAccount;
use platform::xianyu::dev_tunnel::{
    read_msg, write_msg, TunnelMsg, TunnelSession, DEV_TUNNEL_ADDR,
};
use tokio::net::TcpStream;
use tracing::{info, warn};

static ENSURED: OnceLock<Result<(), String>> = OnceLock::new();

/// 确保开发态 Channel Host 已在 `10050` 监听。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 返回值
///
/// Host 已健康或成功拉起时返回 `Ok(())`。
pub fn ensure_dev_channel_host() -> Result<(), String> {
    ENSURED
        .get_or_init(|| {
            if tunnel_port_open() {
                info!(addr = DEV_TUNNEL_ADDR, "复用已运行的 Channel Host");
                return Ok(());
            }
            spawn_detached_host()?;
            wait_until_open(Duration::from_secs(20))?;
            info!(addr = DEV_TUNNEL_ADDR, "Channel Host 已就绪");
            Ok(())
        })
        .clone()
}

/// Tauri 启动后：对 Host 上已有上游会话自动 `dispatcher.connect`（Open 复用，不 `/reg`）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 参数
///
/// * `dispatcher` — 本地调度器（工厂须为隧道版 `XianyuChannel`）
/// * `accounts` — `(account_id, credential, name)` 来自业务库
pub async fn reattach_host_sessions(
    dispatcher: Arc<ChannelDispatcher>,
    accounts: Vec<(String, String, String)>,
) {
    let Ok(sessions) = list_host_sessions().await else {
        warn!("无法列出 Channel Host 会话，跳过自动附着");
        return;
    };
    if sessions.is_empty() {
        info!("Channel Host 无可复用上游会话");
        return;
    }
    let mut attached = 0u32;
    for session in sessions {
        let Some((_, credential, name)) = accounts
            .iter()
            .find(|(id, _, _)| id == &session.account_id)
            .cloned()
        else {
            warn!(
                account = %session.account_id,
                "Host 有会话但本地无账号 Cookie，跳过"
            );
            continue;
        };
        let channel_account = ChannelAccount {
            id: session.account_id.clone(),
            kind: "xianyu".into(),
            name,
            credential,
            enabled: true,
        };
        match dispatcher.connect(&channel_account).await {
            Ok(()) => {
                attached += 1;
                info!(
                    account = %session.account_id,
                    host_state = %session.state,
                    "已经隧道附着 Host 上游（未重新握手）"
                );
            }
            Err(error) => {
                warn!(account = %session.account_id, %error, "自动附着失败");
            }
        }
    }
    info!(attached, "Channel Host 自动附着完成");
}

async fn list_host_sessions() -> Result<Vec<TunnelSession>, String> {
    let mut stream = TcpStream::connect(DEV_TUNNEL_ADDR)
        .await
        .map_err(|error| error.to_string())?;
    let (mut reader, mut writer) = stream.split();
    write_msg(&mut writer, &TunnelMsg::List)
        .await
        .map_err(|error| error.to_string())?;
    match read_msg(&mut reader)
        .await
        .map_err(|error| error.to_string())?
    {
        Some(TunnelMsg::ListOk { sessions }) => Ok(sessions),
        Some(TunnelMsg::Error { message }) => Err(message),
        other => Err(format!("list 响应异常: {other:?}")),
    }
}

fn tunnel_port_open() -> bool {
    std::net::TcpStream::connect(DEV_TUNNEL_ADDR).is_ok()
}

fn wait_until_open(timeout: Duration) -> Result<(), String> {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if tunnel_port_open() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    Err(format!("等待 Channel Host 超时（{DEV_TUNNEL_ADDR}）"))
}

fn resolve_host_exe() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("DINGDA_DEV_CHANNEL_EXE") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("DINGDA_DEV_CHANNEL_EXE 不存在: {}", path.display()));
    }
    let file_name = if cfg!(windows) {
        "dingda-dev-channel.exe"
    } else {
        "dingda-dev-channel"
    };
    if let Ok(current) = std::env::current_exe() {
        if let Some(dir) = current.parent() {
            let candidate = dir.join(file_name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for rel in [
        "../../../target/x86_64-pc-windows-msvc/debug",
        "../../../target/debug",
        "../../../target/x86_64-pc-windows-msvc/release",
        "../../../target/release",
    ] {
        let candidate = manifest.join(rel).join(file_name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "未找到 {file_name}，请先: cargo build -p dingda --bin dingda-dev-channel --target x86_64-pc-windows-msvc"
    ))
}

fn spawn_detached_host() -> Result<(), String> {
    let exe = resolve_host_exe()?;
    info!(path = %exe.display(), "拉起 Channel Host（detach）");
    let mut command = Command::new(&exe);
    command.current_dir(
        exe.parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".")),
    );
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    command
        .spawn()
        .map_err(|error| format!("启动 Channel Host 失败: {error}"))?;
    Ok(())
}
