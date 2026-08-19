//! 闲鱼风控判定 — token / mtop 验证码拦截。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-19

/// 判断文本是否为闲鱼风控拦截（验证码 / 签名异常 / 频率限制）。
///
/// 典型响应：`FAIL_SYS_USER_VALIDATE` + `RGV587_ERROR::SM::哎哟喂,被挤爆啦`，
/// `data.url` 指向 `punish?...action=captcha`。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
///
/// # 参数
/// - `text` — 错误或响应原文
///
/// # 返回值
/// 命中风控关键字返回 `true`。
pub fn is_risk_control_text(text: &str) -> bool {
    [
        "FAIL_SYS_USER_VALIDATE",
        "RGV587",
        "USER_VALIDATE",
        "punish",
        "captcha",
        "被挤爆",
        "FAIL_SYS_ILLEGAL_ACCESS",
    ]
    .iter()
    .any(|keyword| text.contains(keyword))
}

/// 从风控错误原文中提取惩罚页 URL（若有）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
///
/// # 参数
/// - `text` — 含 JSON 或纯 URL 的错误原文
///
/// # 返回值
/// 含 `punish` 或 `captcha` 的 `https://` URL；没有则 `None`。
pub fn extract_punish_url(text: &str) -> Option<String> {
    let start = text.find("https://")?;
    let rest = &text[start..];
    let end = rest
        .find(|ch: char| ch == '"' || ch == ' ' || ch == '\\' || ch == '}' || ch == '\'')
        .unwrap_or(rest.len());
    let url = rest[..end].replace("\\/", "/");
    if url.contains("punish") || url.contains("captcha") {
        Some(url)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risk_control_text_matches_rgv587() {
        assert!(is_risk_control_text(
            r#"token 接口未成功: {"ret":["FAIL_SYS_USER_VALIDATE","RGV587_ERROR::SM::哎哟喂,被挤爆啦"]}"#
        ));
        assert!(!is_risk_control_text("FAIL_SYS_SESSION_EXPIRED"));
    }

    #[test]
    fn extract_punish_url_from_token_error() {
        let text = r#"token 接口未成功: {"data":{"url":"https://h5api.m.goofish.com:443//h5/mtop.taobao.idlemessage.pc.login.token/1.0/_____tmd_____/punish?x5secdata=abc&action=captcha"},"ret":["FAIL_SYS_USER_VALIDATE"]}"#;
        let url = extract_punish_url(text).expect("url");
        assert!(url.contains("punish"));
        assert!(url.starts_with("https://"));
        assert!(!url.contains('"'));
    }
}
