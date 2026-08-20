//! 闲鱼 IM 会话同步 — 拉取未读会话列表。
//!
//! 接口：`mtop.taobao.idlemessage.pc.session.sync` v3.0（参考 goofish-cli `message list-chats`）。
//! 发现登记：[`skills/dingda/guides/xianyu-mtop-discovery.md`](../../../../skills/dingda/guides/xianyu-mtop-discovery.md)
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-20

use common::DingDaResult;
use serde_json::Value;

use super::mtop::{MtopClient, MtopRequest};

/// 同步得到的会话摘要（含未读）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-20
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSummary {
    /// 会话 ID（sessionId）。
    pub session_id: String,
    /// 对端 userId。
    pub peer_id: String,
    /// 对端昵称。
    pub peer_name: String,
    /// 未读条数。
    pub unread: u32,
    /// 最后一条消息摘要。
    pub last_msg: String,
    /// 最后消息时间戳（毫秒）。
    pub ts_ms: i64,
    /// 会话类型（1=真人，3=系统，6=互动，23=通知）。
    pub session_type: i64,
    /// 关联商品 ID（可能为空）。
    pub item_id: String,
}

/// 拉取会话列表（默认最多 50 条）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-20
///
/// # 参数
///
/// * `cookie_str` — 账号 Cookie 原文
/// * `fetch_num` — 拉取条数（默认 50）
///
/// # 返回值
///
/// 成功返回 `(会话列表, 最新 Cookie)`。
pub async fn fetch_sessions(
    cookie_str: &str,
    fetch_num: u32,
) -> DingDaResult<(Vec<SessionSummary>, String)> {
    let limit = if fetch_num == 0 { 50 } else { fetch_num };
    let client = MtopClient::new(cookie_str)?;
    let request = MtopRequest::new(
        "mtop.taobao.idlemessage.pc.session.sync",
        "3.0",
        serde_json::json!({ "fetchNum": limit }),
    )
    .with_param("spm_cnt", "a21ybx.im.0.0");

    let response = client.call(&request).await?;
    if !response.success() {
        return Err(format!("会话同步接口未成功: {}", response.ret).into());
    }

    let sessions = response
        .data()
        .and_then(|data| data.get("sessions"))
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(parse_session_item).collect())
        .unwrap_or_default();

    let cookie = client.cookie().await;
    Ok((sessions, cookie))
}

/// 拉取未读会话（`unread > 0`）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-20
pub async fn fetch_unread_sessions(
    cookie_str: &str,
    fetch_num: u32,
) -> DingDaResult<(Vec<SessionSummary>, String)> {
    let (sessions, cookie) = fetch_sessions(cookie_str, fetch_num).await?;
    let unread: Vec<SessionSummary> = sessions.into_iter().filter(|s| s.unread > 0).collect();
    Ok((unread, cookie))
}

/// 解析单条 session.sync 响应项。
fn parse_session_item(item: &Value) -> Option<SessionSummary> {
    let session = item.get("session")?;
    let user_info = session.get("userInfo").unwrap_or(&Value::Null);
    let message = item.get("message").unwrap_or(&Value::Null);
    // summary 在部分响应里是嵌套对象（含 unread/summary/ts），部分平台直接平铺在 message。
    let summary = message
        .get("summary")
        .filter(|value| value.is_object())
        .unwrap_or(&Value::Null);

    let session_id = json_str(session.get("sessionId"))?;
    if session_id.is_empty() {
        return None;
    }

    let peer_id = json_str(user_info.get("userId")).unwrap_or_default();
    let peer_name = json_str(user_info.get("fishNick"))
        .or_else(|| json_str(user_info.get("nick")))
        .unwrap_or_default();

    // 未读数：message.summary.unread → message.unread → session.unread。
    let unread = [summary, message, session]
        .iter()
        .find_map(|node| node.get("unread").and_then(json_u32))
        .unwrap_or(0);
    // 最后一条消息：message.summary.summary/content → message.content/summary/text。
    let last_msg = json_str(summary.get("summary"))
        .or_else(|| json_str(summary.get("content")))
        .or_else(|| json_str(message.get("content")))
        .or_else(|| json_str(message.get("summary")))
        .or_else(|| json_str(message.get("text")))
        .unwrap_or_default();
    // 时间戳：message.summary.ts → message.ts → session.ts。
    let ts_ms = [summary, message, session]
        .iter()
        .find_map(|node| node.get("ts").and_then(json_i64))
        .unwrap_or(0);
    let session_type = session.get("sessionType").and_then(json_i64).unwrap_or(0);

    let item_id = json_str(session.get("itemId"))
        .or_else(|| json_str(session.pointer("/extension/itemId")))
        .unwrap_or_default();

    Some(SessionSummary {
        session_id,
        peer_id,
        peer_name,
        unread,
        last_msg,
        ts_ms,
        session_type,
        item_id,
    })
}

fn json_str(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) if !text.is_empty() => Some(text.clone()),
        Some(Value::Number(number)) => Some(number.to_string()),
        _ => None,
    }
}

fn json_u32(value: &Value) -> Option<u32> {
    match value {
        Value::Number(number) => number.as_u64().map(|n| n as u32),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn json_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_session_with_unread() {
        let item = json!({
            "session": {
                "sessionId": "123456789@goofish",
                "sessionType": 1,
                "userInfo": {
                    "userId": "99887766",
                    "fishNick": "买家小明"
                }
            },
            "message": {
                "summary": {
                    "unread": 2,
                    "summary": "还在吗？",
                    "ts": 1700000000123_i64
                }
            }
        });
        let parsed = parse_session_item(&item).expect("parse");
        assert_eq!(parsed.peer_id, "99887766");
        assert_eq!(parsed.peer_name, "买家小明");
        assert_eq!(parsed.unread, 2);
        assert_eq!(parsed.last_msg, "还在吗？");
        assert_eq!(parsed.ts_ms, 1700000000123);
        assert_eq!(parsed.session_type, 1);
    }

    #[test]
    fn filters_empty_session_id() {
        let item = json!({
            "session": { "sessionId": "" },
            "message": { "summary": { "unread": 1, "summary": "hi" } }
        });
        assert!(parse_session_item(&item).is_none());
    }
}
