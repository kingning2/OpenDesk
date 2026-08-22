//! 历史消息解析 — 从 `listUserMessages` 响应抽出 [`HistoryMessage`]。
//!
//! 字段参考 goofish-cli `core/ws.py`；由 [`crate::xianyu::core::ws`] 在收帧后调用。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-21

use base64::Engine;
use serde_json::Value;

use crate::protocol::HistoryMessage;

/// 解析 `userMessageModels[]` 单条为历史消息。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 参数
/// - `model` — `userMessageModels` 数组元素
///
/// # 返回值
/// 可解码出正文时返回 [`HistoryMessage`]；缺字段则 `None`。
pub fn parse_history_message(model: &Value) -> Option<HistoryMessage> {
    let message = model.get("message")?;
    let extension = message.get("extension").unwrap_or(&Value::Null);
    let sender_user_id = extension
        .get("senderUserId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let sender_user_name = extension
        .get("reminderTitle")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let data = message
        .pointer("/content/custom/data")
        .and_then(Value::as_str)?;
    let content = decode_history_content(data).unwrap_or_default();
    let created_at_ms = ["createTime", "ts", "createTimeMs"]
        .iter()
        .find_map(|key| message.get(*key).and_then(Value::as_i64))
        .unwrap_or(0);
    Some(HistoryMessage {
        sender_user_id,
        sender_user_name,
        content,
        created_at_ms,
    })
}

/// base64 → JSON 解码消息正文（`content.custom.data`）。
///
/// 解码后可能是字符串，也可能是 `{"text": {"text": "..."}}` / `{"content": "..."}` 等结构。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 参数
/// - `data_base64` — 协议侧 base64 载荷
///
/// # 返回值
/// 可读文本；解码失败返回 `None`。
pub fn decode_history_content(data_base64: &str) -> Option<String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_base64)
        .ok()?;
    let text = String::from_utf8(bytes).ok()?;
    let trimmed = text.trim().to_string();
    let Ok(value) = serde_json::from_str::<Value>(&trimmed) else {
        return Some(trimmed);
    };
    let extract = |node: &Value| -> Option<String> {
        match node {
            Value::String(s) => Some(s.clone()),
            Value::Object(_) => node
                .get("text")
                .and_then(|t| match t {
                    Value::String(s) => Some(s.clone()),
                    Value::Object(_) => {
                        t.get("text").and_then(Value::as_str).map(|s| s.to_string())
                    }
                    _ => None,
                })
                .or_else(|| {
                    node.get("content")
                        .and_then(Value::as_str)
                        .map(|s| s.to_string())
                })
                .or_else(|| {
                    node.get("title")
                        .and_then(Value::as_str)
                        .map(|s| s.to_string())
                }),
            _ => None,
        }
    };
    extract(&value).or(Some(trimmed))
}
