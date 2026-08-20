//! 出站帧构造 — /reg 注册、同步 ack、消息发送。
//!
//! 全部为 JSON 文本帧，对齐闲鱼钉钉系 WebSocket 协议（参考 XianyuAutoAgent）。

use base64::Engine;
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

use super::cookie::parse_cookies;
use super::sign::REG_APP_KEY;

/// 当前毫秒时间戳。
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

/// 生成随机 mid（与闲鱼同构：随机段 + 毫秒时间戳）。
pub fn generate_mid() -> String {
    let random_part = rand::random::<u32>() % 1000;
    format!("{random_part}{} 0", now_ms())
}

/// 生成随机 uuid。
pub fn generate_uuid() -> String {
    format!("-{}1", now_ms())
}

/// 生成设备 id：UUID 样式 + 用户 id 后缀。
pub fn generate_device_id(user_id: &str) -> String {
    let mut result = String::with_capacity(36);
    let chars = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    for i in 0..36 {
        match i {
            8 | 13 | 18 | 23 => result.push('-'),
            14 => result.push('4'),
            19 => {
                let v = rand::random::<u8>();
                result.push(chars[(v & 0x0f) as usize | 0x08] as char);
            }
            _ => {
                let v = rand::random::<u8>();
                result.push(chars[(v % 16) as usize] as char);
            }
        }
    }
    format!("{result}-{user_id}")
}

/// /reg 注册帧（建立连接后首帧）。
pub fn register_frame(device_id: &str, token: &str) -> Value {
    json!({
        "lwp": "/reg",
        "headers": {
            "cache-header": "app-key token ua wv",
            "app-key": REG_APP_KEY,
            "token": token,
            "ua": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36 DingTalk(2.1.5) OS(Windows/10) Browser(Chrome/133.0.0.0) DingWeb/2.1.5",
            "dt": "j",
            "wv": "im:3,au:3,sy:6",
            "sync": "0,0;0;0;",
            "did": device_id,
            "mid": generate_mid()
        }
    })
}

/// 同步 ack 帧（注册后确认 diff）。
///
/// `pts=0`（对齐 goofish-cli）：请求从开头全量同步，服务器才会把历史会话/消息推上来；
/// 用当前时间戳则只同步连接后的新消息，历史会话全部缺失。
pub fn sync_ack_frame() -> Value {
    json!({
        "lwp": "/r/SyncStatus/ackDiff",
        "headers": { "mid": generate_mid() },
        "body": [{
            "pipeline": "sync",
            "tooLong2Tag": "PNM,1",
            "channel": "sync",
            "topic": "sync",
            "highPts": 0,
            "pts": 0,
            "seq": 0,
            "timestamp": now_ms()
        }]
    })
}

/// 心跳帧。
pub fn heartbeat_frame() -> Value {
    json!({
        "lwp": "/!",
        "headers": { "mid": generate_mid() }
    })
}

/// 通用 ACK 响应帧（回显服务端 mid/sid 及关键头）。
#[allow(dead_code)]
pub fn ack_frame(headers: &Value) -> Value {
    let mut out_headers = json!({
        "mid": headers.get("mid").cloned().unwrap_or_else(|| Value::String(generate_mid())),
        "sid": headers.get("sid").cloned().unwrap_or(Value::String(String::new()))
    });
    if let Some(obj) = out_headers.as_object_mut() {
        for key in ["app-key", "ua", "dt"] {
            if let Some(value) = headers.get(key) {
                obj.insert(key.to_string(), value.clone());
            }
        }
    }
    json!({ "code": 200, "headers": out_headers })
}

/// 发送聊天消息帧（`/r/MessageSend/sendByReceiverScope`）。
///
/// `my_id` 为当前账号 `unb`；`cid` 为会话 id（不带 `@goofish` 后缀时自动补）。
pub fn send_message_frame(cid: &str, to_id: &str, my_id: &str, text: &str) -> Value {
    let cid = normalize_peer(cid, "goofish");
    let to = normalize_peer(to_id, "goofish");
    let mine = normalize_peer(my_id, "goofish");

    let content_json = json!({ "contentType": 1, "text": { "text": text } });
    let content_base64 = base64::engine::general_purpose::STANDARD.encode(content_json.to_string());

    json!({
        "lwp": "/r/MessageSend/sendByReceiverScope",
        "headers": { "mid": generate_mid() },
        "body": [
            {
                "uuid": generate_uuid(),
                "cid": cid,
                "conversationType": 1,
                "content": {
                    "contentType": 101,
                    "custom": { "type": 1, "data": content_base64 }
                },
                "redPointPolicy": 0,
                "extension": { "extJson": "{}" },
                "ctx": { "appVersion": "1.0", "platform": "web" },
                "mtags": {},
                "msgReadStatusSetting": 1
            },
            { "actualReceivers": [ to, mine ] }
        ]
    })
}

/// 把 `id` 规范为 `id@domain`；已带后缀则原样返回。
pub fn normalize_peer(id: &str, domain: &str) -> String {
    if id.contains('@') {
        id.to_string()
    } else {
        format!("{id}@{domain}")
    }
}

/// 拉取会话消息历史帧（`/r/MessageManager/listUserMessages`，WebSocket LWP）。
///
/// 帧格式参考 goofish-cli `core/ws.py` `list_user_messages`：
/// body 数组 `[cid@goofish, 向后翻页?, nextCursor, limit, false]`；
/// 首次 `cursor` 传超大值，翻页时用响应里的 `nextCursor` 替换。
pub fn list_user_messages_frame(cid: &str, cursor: i64, limit: u32) -> Value {
    json!({
        "lwp": "/r/MessageManager/listUserMessages",
        "headers": { "mid": generate_mid() },
        "body": [ normalize_peer(cid, "goofish"), false, cursor, limit, false ]
    })
}

/// 从入站 `reminderUrl` 提取 `itemId=xxx`。
pub fn extract_item_id(url: &str) -> Option<String> {
    let after = url.split("itemId=").nth(1)?;
    Some(
        after
            .split('&')
            .next()
            .unwrap_or_default()
            .trim()
            .to_string(),
    )
}

/// 从入站会话标识 `xxx@goofish` 提取裸会话 id。
pub fn extract_cid(raw: &str) -> String {
    raw.split('@').next().unwrap_or(raw).to_string()
}

/// 便捷：由 cookie 字符串生成设备 id。
pub fn device_id_from_cookie(cookie_str: &str) -> Option<String> {
    let cookies = parse_cookies(cookie_str);
    let my_id = cookies.get("unb")?;
    Some(generate_device_id(my_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_frame_has_reg_lwp() {
        let frame = register_frame("did-1", "U-1");
        assert_eq!(frame["lwp"], "/reg");
        assert_eq!(frame["headers"]["did"], "did-1");
    }

    #[test]
    fn send_frame_structure() {
        let frame = send_message_frame("cid1", "peer1", "U-1", "你好");
        assert_eq!(frame["lwp"], "/r/MessageSend/sendByReceiverScope");
        assert_eq!(frame["body"][0]["cid"], "cid1@goofish");
        assert_eq!(frame["body"][1]["actualReceivers"][0], "peer1@goofish");
        // content.data 为 base64 JSON
        let raw = frame["body"][0]["content"]["custom"]["data"]
            .as_str()
            .expect("base64 string");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(raw)
            .expect("valid base64");
        let parsed: Value = serde_json::from_slice(&decoded).expect("valid json");
        assert_eq!(parsed["text"]["text"], "你好");
    }

    #[test]
    fn item_id_extraction() {
        assert_eq!(
            extract_item_id("https://x/?itemId=12345&from=im"),
            Some("12345".to_string())
        );
        assert_eq!(extract_item_id("https://x/?noitem=1"), None);
    }

    #[test]
    fn list_user_messages_frame_structure() {
        let frame = list_user_messages_frame("cid1", 9007199254740991, 20);
        assert_eq!(frame["lwp"], "/r/MessageManager/listUserMessages");
        assert_eq!(frame["body"][0], "cid1@goofish");
        assert_eq!(frame["body"][2], 9007199254740991_i64);
        assert_eq!(frame["body"][3], 20);
        assert!(frame["headers"]["mid"]
            .as_str()
            .is_some_and(|m| !m.is_empty()));
    }

    #[test]
    fn peer_normalization() {
        assert_eq!(normalize_peer("abc", "goofish"), "abc@goofish");
        assert_eq!(normalize_peer("abc@goofish", "goofish"), "abc@goofish");
    }
}
