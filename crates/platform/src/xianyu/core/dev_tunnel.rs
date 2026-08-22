//! 开发态 Channel Host 本地帧隧道协议（长度前缀 JSON）。
//!
//! Host 持有上游闲鱼 WSS；Tauri 经 `127.0.0.1:10050` 附着后取得协议所有权。
//! 帧：`u32` 大端长度 + UTF-8 JSON。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-21

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// 隧道固定端口。
pub const DEV_TUNNEL_PORT: u16 = 10050;

/// 隧道基址。
pub const DEV_TUNNEL_ADDR: &str = "127.0.0.1:10050";

/// 单帧最大载荷（防异常膨胀）。
const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

/// 隧道控制 / 数据报文。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum TunnelMsg {
    /// 首次建连：Host 完成 token + WSS + `/reg`，随后进入转发。
    Open {
        account_id: String,
        credential: String,
    },
    /// 复用已有上游：不重新握手。
    Attach { account_id: String },
    /// Host 就绪，后续为双向 `Text`。
    Ready { account_id: String },
    /// 上游 ↔ 本地 的闲鱼 WS 文本帧。
    Text { data: String },
    /// 列出 Host 上的上游会话。
    List,
    /// [`List`] 响应。
    ListOk { sessions: Vec<TunnelSession> },
    /// 主动断开隧道（上游仍由 Host 保活）。
    Detach,
    /// 错误。
    Error { message: String },
}

/// Host 会话摘要。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelSession {
    /// 账号 id。
    pub account_id: String,
    /// `idle`（Host 保活）或 `attached`（Tauri 持有协议所有权）。
    pub state: String,
}

/// 写出一条隧道报文。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 参数
/// - `writer` — 异步写端
/// - `msg` — 报文
///
/// # 返回值
/// 成功返回 `Ok(())`。
pub async fn write_msg<W: AsyncWrite + Unpin>(
    writer: &mut W,
    msg: &TunnelMsg,
) -> std::io::Result<()> {
    let bytes = serde_json::to_vec(msg)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "tunnel frame too large",
        ));
    }
    let len = (bytes.len() as u32).to_be_bytes();
    writer.write_all(&len).await?;
    writer.write_all(&bytes).await?;
    writer.flush().await?;
    Ok(())
}

/// 读取一条隧道报文。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 参数
/// - `reader` — 异步读端
///
/// # 返回值
/// 成功返回报文；对端关闭返回 `None`。
pub async fn read_msg<R: AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<Option<TunnelMsg>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }
    let len = u32::from_be_bytes(len_buf) as usize;
    if len == 0 || len > MAX_FRAME_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid tunnel frame len={len}"),
        ));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    let msg = serde_json::from_slice(&buf)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    Ok(Some(msg))
}
