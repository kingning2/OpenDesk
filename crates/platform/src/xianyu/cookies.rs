//! 闲鱼 cookies 辅助 — 从结构化 cookies 数组构造会话。
//!
//! 登录载体为 Chrome 扩展导出的快照：Playwright 登录后导出 cookies 数组，
//! Rust 侧将其转为 cookie 字符串并提取协议收发所需的字段。

use common::contracts::ChannelCookie;

/// 从 cookies 数组构造 `name=value; ...` 字符串（供 HTTP/WS 请求头使用）。
pub fn cookies_to_string(cookies: &[ChannelCookie]) -> String {
    cookies
        .iter()
        .filter(|cookie| !cookie.name.is_empty() && !cookie.value.is_empty())
        .map(|cookie| format!("{}={}", cookie.name, cookie.value))
        .collect::<Vec<_>>()
        .join("; ")
}

/// 从凭据（`ChannelAccount.credential`）解析 cookies 数组。
///
/// 兼容三种形态：
/// - **快照 JSON**（Chrome 扩展导出）：`{"cookies":[{...}], "env":{...}, ...}` → 取 `cookies`
/// - **cookies 数组 JSON**（登录后导出）：`[{name,value,...}, ...]`
/// - **旧 cookie 字符串**：`unb=...; _m_h5_tk=...` → 拆成无 domain 的 cookie
pub fn parse_credential(credential: &str) -> Vec<ChannelCookie> {
    // 尝试 JSON 解析。
    let parsed: serde_json::Value = serde_json::from_str(credential).unwrap_or_default();
    if parsed.is_object() {
        if let Some(cookies) = parsed.get("cookies").and_then(serde_json::Value::as_array) {
            return cookies
                .iter()
                .filter_map(|value| serde_json::from_value(value.clone()).ok())
                .collect();
        }
    }
    if parsed.is_array() {
        return parsed
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|value| serde_json::from_value(value.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();
    }
    // 旧 cookie 字符串。
    credential
        .split(';')
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            let (name, value) = part.split_once('=')?;
            Some(ChannelCookie {
                name: name.trim().to_string(),
                value: value.trim().to_string(),
                domain: String::new(),
                path: String::new(),
                expires: None,
                http_only: None,
                secure: None,
                same_site: None,
            })
        })
        .collect()
}

/// 取用户 id（`unb` cookie）。
pub fn my_id(cookies: &[ChannelCookie]) -> Option<String> {
    cookies
        .iter()
        .find(|cookie| cookie.name == "unb")
        .map(|cookie| cookie.value.clone())
}

/// 从 cookies 数组构造设备 id（UUID 样式 + 用户 id 后缀）。
pub fn device_id(cookies: &[ChannelCookie]) -> Option<String> {
    let unb = my_id(cookies)?;
    Some(super::message::generate_device_id(&unb))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cookies() -> Vec<ChannelCookie> {
        vec![
            ChannelCookie {
                name: "unb".into(),
                value: "U-123".into(),
                domain: common::constants::xianyu::COOKIE_DOMAIN.into(),
                path: "/".into(),
                expires: None,
                http_only: Some(false),
                secure: Some(false),
                same_site: Some("Lax".into()),
            },
            ChannelCookie {
                name: "_m_h5_tk".into(),
                value: "token_rest".into(),
                domain: common::constants::xianyu::COOKIE_DOMAIN.into(),
                path: "/".into(),
                expires: None,
                http_only: Some(true),
                secure: Some(false),
                same_site: None,
            },
            ChannelCookie {
                name: "".into(),
                value: "".into(),
                domain: "".into(),
                path: "".into(),
                expires: None,
                http_only: None,
                secure: None,
                same_site: None,
            },
        ]
    }

    #[test]
    fn cookies_to_string_joins_non_empty() {
        let str = cookies_to_string(&sample_cookies());
        assert!(str.contains("unb=U-123"));
        assert!(str.contains("_m_h5_tk=token_rest"));
        assert!(!str.contains("; ;")); // 空 cookie 被过滤
    }

    #[test]
    fn extracts_my_id() {
        let cookies = sample_cookies();
        assert_eq!(my_id(&cookies), Some("U-123".to_string()));
    }

    #[test]
    fn device_id_from_cookies() {
        let cookies = sample_cookies();
        let id = device_id(&cookies).expect("device id");
        assert!(id.ends_with("-U-123"));
        assert!(id.contains('-'));
    }

    #[test]
    fn parse_credential_handles_all_forms() {
        // 旧 cookie 字符串。
        let legacy = parse_credential("unb=U-1; _m_h5_tk=tk_x; cookie2=c2");
        assert_eq!(my_id(&legacy), Some("U-1".to_string()));

        // cookies 数组 JSON。
        let array_json = r#"[{"name":"unb","value":"U-9","domain":".goofish.com","path":"/"}]"#;
        let from_array = parse_credential(array_json);
        assert_eq!(my_id(&from_array), Some("U-9".to_string()));

        // 快照 JSON（含 cookies 数组）。
        let snapshot_json = format!(
            r#"{{"capturedAt":"2026-08-11","env":{{}},"cookies":{},"headers":{{}}}}"#,
            array_json
        );
        let from_snapshot = parse_credential(&snapshot_json);
        assert_eq!(my_id(&from_snapshot), Some("U-9".to_string()));
    }
}
