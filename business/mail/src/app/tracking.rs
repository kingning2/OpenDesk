//! Outbound open-tracking pixel helpers (email-agent parity).
//!
//! 作者：coisini
//! 创建时间：2026-07-22

use ports::mail::MailEmailReadIntegrationConfig;

/// Generate a 32-char hex tracking id (same shape as email-agent).
///
/// 作者：coisini
/// 创建时间：2026-07-22
pub fn make_tracking_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

/// Whether remote open tracking is enabled from UI settings.
///
/// 作者：coisini
/// 创建时间：2026-08-01
pub fn integration_enabled(config: &MailEmailReadIntegrationConfig) -> bool {
    config.enabled && !config.api_base.trim().is_empty()
}

/// Append a 1×1 hidden tracking pixel to HTML (before `</body>` or at end).
///
/// 作者：coisini
/// 创建时间：2026-07-22
pub fn append_open_tracking_pixel(
    config: &MailEmailReadIntegrationConfig,
    html: &str,
    tracking_id: &str,
    recipient_email: &str,
) -> String {
    if !integration_enabled(config) {
        return html.to_string();
    }
    if tracking_id.is_empty() {
        return html.to_string();
    }

    let src = build_integration_url(
        config,
        &config.pixel_path_template,
        recipient_email,
        tracking_id,
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
/// 作者：coisini
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
/// 作者：coisini
/// 创建时间：2026-07-22
pub fn prepare_tracked_html(
    config: &MailEmailReadIntegrationConfig,
    body_text: &str,
    body_html: Option<&str>,
    tracking_id: &str,
    recipient_email: &str,
) -> Option<String> {
    let base_html = prepare_outbound_html(body_text, body_html)?;
    Some(append_open_tracking_pixel(
        config,
        &base_html,
        tracking_id,
        recipient_email,
    ))
}

/// Query remote email-read API for recipient open events.
///
/// 作者：coisini
/// 创建时间：2026-08-01
pub fn fetch_open_status(
    config: &MailEmailReadIntegrationConfig,
    recipient_email: &str,
    tracking_id: &str,
) -> Option<(Option<String>, i64)> {
    if !integration_enabled(config) {
        return None;
    }
    let recipient = recipient_email.trim();
    let tracking_id = tracking_id.trim();
    if recipient.is_empty() || tracking_id.is_empty() {
        return None;
    }

    let url = build_integration_url(config, &config.query_path_template, recipient, tracking_id);

    let response = reqwest::blocking::get(&url)
        .map_err(|error| tracing::warn!(%error, "mail.open_status.request_failed"))
        .ok()?;
    if !response.status().is_success() {
        tracing::warn!(status = %response.status(), "mail.open_status.bad_status");
        return None;
    }

    let payload = response
        .json::<serde_json::Value>()
        .map_err(|error| tracing::warn!(%error, "mail.open_status.parse_failed"))
        .ok()?;

    parse_open_payload(&payload)
}

/// Probe integration query URL and return raw JSON for the settings UI test runner.
///
/// 作者：coisini
/// 创建时间：2026-08-01
pub fn probe_open_status(
    config: &MailEmailReadIntegrationConfig,
    recipient_email: &str,
    tracking_id: &str,
) -> Result<String, String> {
    if !integration_enabled(config) {
        return Err("mail.integration.disabled".to_string());
    }
    let recipient = recipient_email.trim();
    let tracking_id = tracking_id.trim();
    if recipient.is_empty() || tracking_id.is_empty() {
        return Err("mail.integration.probe_params_invalid".to_string());
    }

    let url = build_integration_url(config, &config.query_path_template, recipient, tracking_id);
    let response = reqwest::blocking::get(&url).map_err(|error| error.to_string())?;
    let status = response.status();
    let body = response.text().map_err(|error| error.to_string())?;
    if !status.is_success() {
        return Err(format!("mail.integration.http_{}: {body}", status.as_u16()));
    }
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err("mail.integration.empty_response".to_string());
    }
    serde_json::from_str::<serde_json::Value>(trimmed)
        .map_err(|error| format!("mail.integration.invalid_json: {error}"))?;
    Ok(body)
}

/// Build absolute URL from base + path template placeholders.
///
/// 作者：coisini
/// 创建时间：2026-08-01
pub fn build_integration_url(
    config: &MailEmailReadIntegrationConfig,
    path_template: &str,
    recipient_email: &str,
    tracking_id: &str,
) -> String {
    let base = config.api_base.trim().trim_end_matches('/');
    let path = path_template
        .replace("{{base}}", base)
        .replace("{{email}}", &url_encode(recipient_email))
        .replace("{{mailId}}", &url_encode(tracking_id));
    if path.starts_with("http://") || path.starts_with("https://") {
        path
    } else if path.starts_with('/') {
        format!("{base}{path}")
    } else {
        format!("{base}/{path}")
    }
}

fn parse_open_payload(payload: &serde_json::Value) -> Option<(Option<String>, i64)> {
    let items = payload.get("items").and_then(|value| value.as_array());
    let Some(items) = items else {
        return Some((None, 0));
    };
    if items.is_empty() {
        return Some((None, 0));
    }
    let open_count = items.len() as i64;
    let opened_at = items.first().and_then(extract_open_timestamp);
    Some((opened_at, open_count))
}

fn extract_open_timestamp(item: &serde_json::Value) -> Option<String> {
    for key in ["opened_at", "open_time", "timestamp", "time", "created_at"] {
        if let Some(value) = item.get(key).and_then(|value| value.as_str()) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
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
    use ports::mail::MailEmailReadIntegrationConfig;

    #[test]
    fn tracking_id_is_32_hex_chars() {
        let id = make_tracking_id();
        assert_eq!(id.len(), 32);
        assert!(id.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn append_pixel_before_body_close() {
        let config = MailEmailReadIntegrationConfig {
            enabled: true,
            api_base: "https://kol-service.gbyte.com".to_string(),
            pixel_path_template: "/api/v1/email-read/pixel?email={{email}}&mailId={{mailId}}"
                .to_string(),
            query_path_template: "/api/v1/email-read?email={{email}}&mailId={{mailId}}".to_string(),
            parse_script: String::new(),
        };
        let html = append_open_tracking_pixel(
            &config,
            "<html><body><p>Hi</p></body></html>",
            "abc123",
            "a@b.com",
        );
        assert!(html.contains("/api/v1/email-read/pixel"));
        assert!(html.contains("mailId=abc123"));
        assert!(html.contains("<p>Hi</p>"));
    }
}
