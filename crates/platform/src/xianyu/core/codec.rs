//! 入站数据帧编解码 — 闲鱼消息通道部分帧为 MessagePack 二进制。
//!
//! 参考 XianyuAutoAgent 的 MessagePack 解码逻辑：入站 `syncPushPackage.data[0].data`
//! 为 base64 包裹的 MessagePack，解码后是数字键导航的字典
//! （如 `msg["1"]["10"]["reminderContent"]`）。

use rmpv::Value;

/// 入站文本消息路径约定（来自闲鱼钉钉协议逆向）。
pub const MSG_TYPE: &[&str] = &["1", "2"]; // `xxx@goofish` 会话标识
pub const MSG_CREATE_TIME: &[&str] = &["1", "5"]; // 毫秒时间戳
pub const MSG_CONTENT: &[&str] = &["1", "10", "reminderContent"];
pub const MSG_SENDER_NAME: &[&str] = &["1", "10", "reminderTitle"];
pub const MSG_SENDER_ID: &[&str] = &["1", "10", "senderUserId"];
pub const MSG_URL: &[&str] = &["1", "10", "reminderUrl"];

/// 从 MessagePack 帧中按字符串键路径取值（跳过 map 类型检查，缺失返回 `None`）。
pub fn get_path<'a>(root: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = root;
    for key in path {
        match current {
            Value::Map(entries) => {
                let key_value = Value::String(key.to_string().into());
                let found = entries.iter().find(|(k, _)| k == &key_value);
                match found {
                    Some((_, value)) => current = value,
                    None => return None,
                }
            }
            _ => return None,
        }
    }
    Some(current)
}

/// 取路径下的字符串值；非字符串返回 `None`。
pub fn get_string(root: &Value, path: &[&str]) -> Option<String> {
    get_path(root, path).and_then(|value| match value {
        Value::String(s) => s.as_str().map(|s| s.to_string()),
        _ => None,
    })
}

/// 取路径下的整数值；数值型返回 `i64`。
pub fn get_i64(root: &Value, path: &[&str]) -> Option<i64> {
    get_path(root, path).and_then(|value| match value {
        Value::Integer(i) => i.as_i64(),
        _ => None,
    })
}

/// 从出站回复流中识别是否包含可导航的聊天消息结构。
/// 返回 `true` 表示该帧属于聊天内容，可调用各 `get_*` 提取字段。
pub fn is_chat_message(root: &Value) -> bool {
    get_path(root, MSG_CONTENT).is_some() && get_path(root, MSG_SENDER_ID).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmpv::Value;

    fn build_sample() -> Value {
        // 构造与闲鱼入站结构一致的 MessagePack 值：
        // { "1": { "2": "cid@goofish", "5": 1700000000123, "10": {
        //     "reminderContent": "你好", "reminderTitle": "买家",
        //     "senderUserId": "peer-1", "reminderUrl": ".../itemId=12345&..." } } }
        Value::Map(vec![(
            Value::String("1".into()),
            Value::Map(vec![
                (
                    Value::String("2".into()),
                    Value::String("cid@goofish".into()),
                ),
                (
                    Value::String("5".into()),
                    Value::Integer(1_700_000_000_123_i64.into()),
                ),
                (
                    Value::String("10".into()),
                    Value::Map(vec![
                        (
                            Value::String("reminderContent".into()),
                            Value::String("你好，多少钱".into()),
                        ),
                        (
                            Value::String("reminderTitle".into()),
                            Value::String("买家A".into()),
                        ),
                        (
                            Value::String("senderUserId".into()),
                            Value::String("peer-1".into()),
                        ),
                        (
                            Value::String("reminderUrl".into()),
                            Value::String("https://.../itemId=12345&from=x".into()),
                        ),
                    ]),
                ),
            ]),
        )])
    }

    #[test]
    fn decode_chat_fields() {
        let frame = build_sample();
        assert!(is_chat_message(&frame));
        assert_eq!(
            get_string(&frame, MSG_CONTENT),
            Some("你好，多少钱".to_string())
        );
        assert_eq!(
            get_string(&frame, MSG_SENDER_NAME),
            Some("买家A".to_string())
        );
        assert_eq!(
            get_string(&frame, MSG_SENDER_ID),
            Some("peer-1".to_string())
        );
        assert_eq!(get_i64(&frame, MSG_CREATE_TIME), Some(1_700_000_000_123));
    }

    #[test]
    fn missing_path_returns_none() {
        let frame = Value::Map(vec![]);
        assert!(!is_chat_message(&frame));
        assert_eq!(get_string(&frame, MSG_CONTENT), None);
    }
}
