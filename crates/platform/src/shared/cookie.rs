//! 闲鱼 Cookie 解析与设备 id 工具。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-21

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// 当前毫秒时间戳。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 返回值
/// Unix 纪元起的毫秒数；时钟异常时返回 `0`。
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

/// 生成设备 id：UUID 样式 + 用户 id 后缀。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 参数
/// - `user_id` — 闲鱼用户 id（通常为 `unb`）
///
/// # 返回值
/// 协议侧 `did` 字段用的设备标识。
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

/// 由 cookie 凭据生成设备 id。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 参数
/// - `cookie_str` — `name=value; ...` 或 JSON cookies / 快照
///
/// # 返回值
/// 含 `unb` 时返回设备 id；否则 `None`。
pub fn device_id_from_cookie(cookie_str: &str) -> Option<String> {
    crate::shared::cookies::device_id(&crate::shared::cookies::parse_credential(cookie_str))
}

/// 解析浏览器导出的 Cookie 凭据为键值表。
///
/// 兼容 `k=v; ...`、cookies JSON 数组、快照 JSON（与 [`crate::shared::cookies::parse_credential`] 一致）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-21
///
/// # 参数
/// - `cookie_str` — 任意凭据形态
///
/// # 返回值
/// Cookie 名 → 值；同名按 goofish > taobao 域优先。
pub fn parse_cookies(cookie_str: &str) -> HashMap<String, String> {
    let header = crate::shared::cookies::credential_to_cookie_header(cookie_str);
    header
        .split(';')
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            let (key, value) = part.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

/// 取用户 id（`unb` cookie）。
pub fn my_id(cookies: &HashMap<String, String>) -> Option<String> {
    cookies.get("unb").cloned()
}

/// 取签名 token（`_m_h5_tk` 的 `_` 前缀）。
pub fn sign_token(cookies: &HashMap<String, String>) -> Option<String> {
    cookies
        .get("_m_h5_tk")
        .map(|value| value.split('_').next().unwrap_or_default().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_cookie_string() {
        let cookies = parse_cookies("unb=U-123; _m_h5_tk=abc_token_987; cookie2=xyz; cna=cnaval");
        assert_eq!(cookies.get("unb"), Some(&"U-123".to_string()));
        assert_eq!(cookies.get("_m_h5_tk"), Some(&"abc_token_987".to_string()));
    }

    #[test]
    fn parse_handles_spaces_and_missing() {
        let cookies = parse_cookies("  a=1 ; b=2");
        assert_eq!(cookies.len(), 2);
        assert_eq!(cookies.get("a"), Some(&"1".to_string()));
        let empty = parse_cookies("");
        assert!(empty.is_empty());
    }

    #[test]
    fn my_id_and_sign_token_extraction() {
        let cookies = parse_cookies("unb=U-9; _m_h5_tk=tokenpart_rest");
        assert_eq!(my_id(&cookies), Some("U-9".to_string()));
        assert_eq!(sign_token(&cookies), Some("tokenpart".to_string()));
    }

    #[test]
    fn parse_cookies_reads_json_array_unb() {
        let json = r#"[{"name":"unb","value":"U-json","domain":".goofish.com","path":"/"}]"#;
        let cookies = parse_cookies(json);
        assert_eq!(cookies.get("unb"), Some(&"U-json".to_string()));
        assert_eq!(
            device_id_from_cookie(json)
                .as_deref()
                .map(|id| id.ends_with("-U-json")),
            Some(true)
        );
    }
}
