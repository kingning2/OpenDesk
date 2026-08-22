//! 入站 `syncPushPackage` 解析 — 对齐 goofish-cli `core/ws.py`。
//!
//! `session.sync` 只返回活跃 Top N；完整会话列表靠 `ackDiff(pts=0)` 后的
//! 推包补齐（`operation.sessionInfo` / `new_msg` 元事件 / 带正文消息）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-21

use base64::Engine;
use serde_json::Value;

use crate::xianyu::message::frames::extract_cid;

/// 从推包解码出的会话骨架（无可靠 peer 时用 cid 占位）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushedSession {
    /// goofish 会话裸 id。
    pub cid: String,
    /// 关联商品 id（可能为空）。
    pub item_id: String,
    /// 商品标题（可能为空）。
    pub item_title: String,
    /// 最近消息时间戳（毫秒字符串，可能为空）。
    pub updated_at: String,
}

/// 从推包解码出的带正文入站消息（新 web 格式）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushedMessage {
    /// goofish 会话裸 id。
    pub cid: String,
    /// 发送者 userId。
    pub peer_id: String,
    /// 发送者昵称。
    pub peer_name: String,
    /// 关联商品 id。
    pub item_id: String,
    /// 文本正文。
    pub content: String,
    /// 毫秒时间戳。
    pub created_at_ms: i64,
}

/// 一帧 `syncPushPackage` 的解析结果。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PushBatch {
    /// 会话激活 / new_msg 骨架。
    pub sessions: Vec<PushedSession>,
    /// 带正文的消息事件。
    pub messages: Vec<PushedMessage>,
}

/// 解析帧 body 内的 `syncPushPackage`。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 参数
/// - `msg` — 完整 LWP JSON 帧
///
/// # 返回值
/// 无推包时返回空 batch；解码失败的条目跳过。
pub fn parse_sync_push_package(msg: &Value) -> PushBatch {
    let Some(data_list) = msg
        .pointer("/body/syncPushPackage/data")
        .and_then(Value::as_array)
    else {
        return PushBatch::default();
    };

    let mut batch = PushBatch::default();
    let mut seen_cid = std::collections::HashSet::new();

    for item in data_list {
        let Some(raw) = item.get("data").and_then(Value::as_str) else {
            continue;
        };
        let Some(decoded) = decode_push_payload(raw) else {
            continue;
        };
        ingest_decoded(&decoded, &mut batch, &mut seen_cid);
    }
    batch
}

/// 三种尝试：明文 JSON → base64(JSON)。老协议 encrypt 暂不接入。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 参数
/// - `raw` — `syncPushPackage.data[].data` 字符串
///
/// # 返回值
/// 解码成功的 JSON 对象；失败返回 `None`。
pub fn decode_push_payload(raw: &str) -> Option<Value> {
    if let Ok(value) = serde_json::from_str::<Value>(raw) {
        return Some(value);
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(raw.trim())
        .ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn ingest_decoded(
    decoded: &Value,
    batch: &mut PushBatch,
    seen_cid: &mut std::collections::HashSet<String>,
) {
    // a) 会话激活：{ sessionId, operation.sessionInfo.extensions }
    let cid_raw = decoded
        .get("sessionId")
        .and_then(Value::as_str)
        .unwrap_or("");
    let sess_info = decoded.pointer("/operation/sessionInfo");
    if !cid_raw.is_empty() && sess_info.is_some() {
        let cid = extract_cid(cid_raw);
        if !cid.is_empty() && seen_cid.insert(cid.clone()) {
            let ext = sess_info
                .and_then(|s| s.get("extensions"))
                .unwrap_or(&Value::Null);
            // 注意：extensions.extUserId 是卖家 id，不能当 peer（对齐 goofish-cli）。
            let item_id = json_str(ext.get("itemId")).unwrap_or_default();
            let item_title = json_str(ext.get("itemTitle")).unwrap_or_default();
            batch.sessions.push(PushedSession {
                cid,
                item_id,
                item_title,
                updated_at: String::new(),
            });
        }
    }

    // b) new_msg 轻量通知：{"1":"cid@goofish","2":1,"3":msgId,"4":ts}
    if let Some(meta_cid) = extract_new_msg_cid(decoded) {
        let ts = decoded
            .get("4")
            .map(|v| match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => String::new(),
            })
            .unwrap_or_default();
        if seen_cid.insert(meta_cid.clone()) {
            batch.sessions.push(PushedSession {
                cid: meta_cid,
                item_id: String::new(),
                item_title: String::new(),
                updated_at: ts,
            });
        } else if !ts.is_empty() {
            if let Some(existing) = batch.sessions.iter_mut().find(|s| s.cid == meta_cid) {
                if existing.updated_at.is_empty() {
                    existing.updated_at = ts;
                }
            }
        }
    }

    // c) 带正文的新格式消息（contentType 1/101）
    if let Some(message) = extract_incoming_text(decoded) {
        batch.messages.push(message);
    }
}

fn extract_new_msg_cid(decoded: &Value) -> Option<String> {
    let one = decoded.get("1").and_then(Value::as_str)?;
    let two = decoded.get("2").and_then(Value::as_i64)?;
    let three = decoded.get("3").and_then(Value::as_str)?;
    if two == 1 && one.ends_with("@goofish") && !three.is_empty() {
        let cid = extract_cid(one);
        if cid.is_empty() {
            None
        } else {
            Some(cid)
        }
    } else {
        None
    }
}

/// 从新 web 格式推包抽出文本消息。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 参数
/// - `decoded` — 单条 push 解码结果
///
/// # 返回值
/// 可识别正文时返回 [`PushedMessage`]；会话激活等无正文事件返回 `None`。
pub fn extract_incoming_text(decoded: &Value) -> Option<PushedMessage> {
    let op = decoded.get("operation")?;
    if !op.is_object() {
        return None;
    }
    let content = op.get("content").unwrap_or(&Value::Null);
    let content_type = content.get("contentType").and_then(Value::as_i64)?;
    // 8 = 会话激活，无聊天正文。
    if content_type == 8 {
        return None;
    }

    let sess = op.get("sessionInfo").unwrap_or(&Value::Null);
    let sender = op.get("senderInfo").unwrap_or(&Value::Null);
    let reminder = content.get("reminder").unwrap_or(&Value::Null);

    let cid_raw = decoded
        .get("sessionId")
        .and_then(Value::as_str)
        .or_else(|| sess.get("sessionId").and_then(Value::as_str))
        .unwrap_or("");
    let cid = extract_cid(cid_raw);
    if cid.is_empty() {
        return None;
    }

    let mut text = String::new();
    if content_type == 1 {
        text = content
            .pointer("/text/text")
            .and_then(Value::as_str)
            .or_else(|| reminder.get("reminderContent").and_then(Value::as_str))
            .unwrap_or("")
            .to_string();
    } else if content_type == 101 {
        if let Some(data_b64) = content.pointer("/custom/data").and_then(Value::as_str) {
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data_b64) {
                if let Ok(payload) = serde_json::from_slice::<Value>(&bytes) {
                    text = payload
                        .pointer("/text/text")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                }
            }
        }
        if text.is_empty() {
            text = reminder
                .get("reminderContent")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
        }
    } else {
        text = reminder
            .get("reminderContent")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
    }

    if text.is_empty() {
        return None;
    }

    let peer_id = json_str(sender.get("senderUserId"))
        .or_else(|| json_str(reminder.get("senderUserId")))
        .unwrap_or_default();
    if peer_id.is_empty() {
        return None;
    }

    let peer_name = json_str(reminder.get("reminderTitle")).unwrap_or_default();
    let item_id = sess
        .get("extensions")
        .and_then(|ext| json_str(ext.get("itemId")))
        .unwrap_or_default();
    let created_at_ms = decoded
        .get("createTime")
        .and_then(Value::as_i64)
        .or_else(|| content.get("createTime").and_then(Value::as_i64))
        .unwrap_or(0);

    Some(PushedMessage {
        cid,
        peer_id,
        peer_name,
        item_id,
        content: text,
        created_at_ms,
    })
}

fn json_str(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) if !text.is_empty() => Some(text.clone()),
        Some(Value::Number(number)) => Some(number.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decodes_plain_and_base64_payload() {
        let plain = r#"{"sessionId":"111@goofish"}"#;
        assert_eq!(
            decode_push_payload(plain).unwrap()["sessionId"],
            "111@goofish"
        );
        let b64 = base64::engine::general_purpose::STANDARD.encode(plain);
        assert_eq!(
            decode_push_payload(&b64).unwrap()["sessionId"],
            "111@goofish"
        );
    }

    #[test]
    fn extracts_session_info_without_using_ext_user_as_peer() {
        let payload = json!({
            "sessionId": "60585751957@goofish",
            "operation": {
                "sessionInfo": {
                    "sessionType": 1,
                    "extensions": {
                        "itemId": "item-9",
                        "itemTitle": "二手手机",
                        "extUserId": "seller-self-id"
                    }
                }
            }
        });
        let frame = json!({
            "body": {
                "syncPushPackage": {
                    "data": [{ "data": payload.to_string() }]
                }
            }
        });
        let batch = parse_sync_push_package(&frame);
        assert_eq!(batch.sessions.len(), 1);
        assert_eq!(batch.sessions[0].cid, "60585751957");
        assert_eq!(batch.sessions[0].item_id, "item-9");
        assert_eq!(batch.sessions[0].item_title, "二手手机");
    }

    #[test]
    fn extracts_new_msg_meta() {
        let payload = json!({
            "1": "cid99@goofish",
            "2": 1,
            "3": "msgid",
            "4": "1700000000000"
        });
        let frame = json!({
            "body": {
                "syncPushPackage": {
                    "data": [{ "data": payload.to_string() }]
                }
            }
        });
        let batch = parse_sync_push_package(&frame);
        assert_eq!(batch.sessions.len(), 1);
        assert_eq!(batch.sessions[0].cid, "cid99");
        assert_eq!(batch.sessions[0].updated_at, "1700000000000");
    }

    #[test]
    fn extracts_text_message() {
        let payload = json!({
            "sessionId": "cid1@goofish",
            "operation": {
                "content": {
                    "contentType": 1,
                    "text": { "text": "还在吗" },
                    "reminder": {
                        "reminderTitle": "买家",
                        "senderUserId": "peer-42",
                        "reminderContent": "还在吗"
                    }
                },
                "senderInfo": { "senderUserId": "peer-42" },
                "sessionInfo": {
                    "extensions": { "itemId": "item-1" }
                }
            }
        });
        let msg = extract_incoming_text(&payload).expect("text");
        assert_eq!(msg.cid, "cid1");
        assert_eq!(msg.peer_id, "peer-42");
        assert_eq!(msg.content, "还在吗");
        assert_eq!(msg.item_id, "item-1");
    }
}
