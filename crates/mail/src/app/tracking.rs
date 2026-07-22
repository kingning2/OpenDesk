//! Outbound open-tracking pixel helpers (email-agent parity).
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-22

use std::env;

/// Remote Email Read API base used by legacy email-agent.
const DEFAULT_EMAIL_READ_API_BASE: &str = "https://kol-service.gbyte.com";

/// Generate a 32-char hex tracking id (same shape as email-agent).
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub fn make_tracking_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

/// Resolve tracking API base URL from env or default.
///
/// Set `OPENDESK_EMAIL_READ_API_BASE=off` to disable remote pixel injection.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub fn email_read_api_base() -> Option<String> {
    match env::var("OPENDESK_EMAIL_READ_API_BASE").as_deref() {
        Ok("off") | Ok("") => None,
        Ok(value) => Some(value.trim_end_matches('/').to_string()),
        Err(_) => Some(DEFAULT_EMAIL_READ_API_BASE.to_string()),
    }
}

/// Append a 1×1 hidden tracking pixel to HTML (before `</body>` or at end).
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub fn append_open_tracking_pixel(html: &str, tracking_id: &str, recipient_email: &str) -> String {
    let Some(base) = email_read_api_base() else {
        return html.to_string();
    };
    if tracking_id.is_empty() {
        return html.to_string();
    }

    let src = format!(
        "{base}/api/v1/email-read/pixel?email={}&mailId={}",
        url_encode(recipient_email),
        url_encode(tracking_id)
    );
    let pixel = format!(
        r#"<img src="{src}" width="1" height="1" alt="" style="display:none;width:1px;height:1px;border:0;opacity:0" />"#
    );
    let body = html;
    if body.to_ascii_lowercase().contains("</body>") {
        body.replace("</body>", &format!("{pixel}</body>"))
            .replace("</BODY>", &format!("{pixel}</BODY>"))
    } else {
        format!("{body}{pixel}")
    }
}

/// Build HTML for SMTP when only plain text was provided (no tracking pixel).
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub fn prepare_outbound_html(body_text: &str, body_html: Option<&str>) -> Option<String> {
    let html = body_html
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| text_to_html(body_text));
    if html.trim().is_empty() {
        return None;
    }
    Some(html)
}

/// Build HTML for SMTP when only plain text was provided, then inject tracking pixel.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub fn prepare_tracked_html(
    body_text: &str,
    body_html: Option<&str>,
    tracking_id: &str,
    recipient_email: &str,
) -> Option<String> {
    let base_html = prepare_outbound_html(body_text, body_html)?;
    Some(append_open_tracking_pixel(
        &base_html,
        tracking_id,
        recipient_email,
    ))
}

fn text_to_html(text: &str) -> String {
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!("<html><body><div style=\"white-space:pre-wrap;\">{escaped}</div></body></html>")
}

fn url_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracking_id_is_32_hex_chars() {
        let id = make_tracking_id();
        assert_eq!(id.len(), 32);
        assert!(id.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn append_pixel_before_body_close() {
        std::env::set_var("OPENDESK_EMAIL_READ_API_BASE", "https://example.com");
        let html =
            append_open_tracking_pixel("<html><body><p>Hi</p></body></html>", "abc123", "a@b.com");
        assert!(html.contains("/api/v1/email-read/pixel"));
        assert!(html.contains("mailId=abc123"));
        assert!(html.contains("<p>Hi</p>"));
        std::env::remove_var("OPENDESK_EMAIL_READ_API_BASE");
    }
}
