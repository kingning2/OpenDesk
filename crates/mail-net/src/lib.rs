//! SMTP transport for outbound business mail.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-22

mod imap;

pub use imap::{fetch_messages_since, watch_inbox_idle, ImapEndpoint, ImapFetchedMessage};

use std::time::Duration;

use lettre::message::{Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

/// SMTP connection settings for one outbound send.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[derive(Debug, Clone)]
pub struct SmtpEndpoint {
    /// SMTP host name.
    pub host: String,
    /// SMTP port (465 implicit TLS / 587 STARTTLS / other).
    pub port: u16,
    /// Whether TLS should be used.
    pub use_tls: bool,
    /// SMTP auth username.
    pub username: String,
    /// SMTP auth password (never log).
    pub password: String,
}

/// Outbound message payload for SMTP.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[derive(Debug, Clone)]
pub struct SmtpOutboundMessage {
    /// From address.
    pub from_address: String,
    /// Optional display name.
    pub from_name: Option<String>,
    /// Recipient address.
    pub to_address: String,
    /// Subject line.
    pub subject: String,
    /// Plain-text body.
    pub body_text: String,
    /// Optional HTML body.
    pub body_html: Option<String>,
}

/// Send one message over SMTP and return the server response summary.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
///
/// # 参数
///
/// * `endpoint` - SMTP host / auth settings
/// * `message` - Outbound mail content
///
/// # 返回值
///
/// * `Ok(summary)` - SMTP accepted the message
/// * `Err(message)` - Connection, auth, or send failure (no password included)
///
/// # 注意事项
///
/// Password must never appear in returned errors or logs.
pub fn send_smtp(endpoint: &SmtpEndpoint, message: &SmtpOutboundMessage) -> Result<String, String> {
    let from = build_mailbox(&message.from_address, message.from_name.as_deref())?;
    let to = build_mailbox(&message.to_address, None)?;

    let builder = Message::builder()
        .from(from)
        .to(to)
        .subject(message.subject.clone());

    let email = if let Some(html) = message
        .body_html
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        builder
            .multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::plain(message.body_text.clone()))
                    .singlepart(SinglePart::html(html.to_string())),
            )
            .map_err(|error| format!("mail.build_failed: {error}"))?
    } else {
        builder
            .singlepart(SinglePart::plain(message.body_text.clone()))
            .map_err(|error| format!("mail.build_failed: {error}"))?
    };

    let mailer = build_transport(endpoint)?;
    let response = mailer
        .send(&email)
        .map_err(|error| sanitize_smtp_error(&error.to_string(), &endpoint.password))?;

    Ok(response
        .message()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string())
}

fn build_mailbox(address: &str, name: Option<&str>) -> Result<Mailbox, String> {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        return Err("mail.address_required".to_string());
    }
    match name.map(str::trim).filter(|value| !value.is_empty()) {
        Some(display) => format!("{display} <{trimmed}>")
            .parse()
            .map_err(|error| format!("mail.address_invalid: {error}")),
        None => trimmed
            .parse()
            .map_err(|error| format!("mail.address_invalid: {error}")),
    }
}

fn build_transport(endpoint: &SmtpEndpoint) -> Result<SmtpTransport, String> {
    let credentials = Credentials::new(endpoint.username.clone(), endpoint.password.clone());
    let timeout = Duration::from_secs(30);

    let builder = if !endpoint.use_tls {
        SmtpTransport::builder_dangerous(&endpoint.host)
            .port(endpoint.port)
            .timeout(Some(timeout))
            .credentials(credentials)
    } else if endpoint.port == 465 {
        SmtpTransport::relay(&endpoint.host)
            .map_err(|error| format!("mail.smtp_tls_init: {error}"))?
            .port(endpoint.port)
            .timeout(Some(timeout))
            .credentials(credentials)
    } else {
        SmtpTransport::starttls_relay(&endpoint.host)
            .map_err(|error| format!("mail.smtp_starttls_init: {error}"))?
            .port(endpoint.port)
            .timeout(Some(timeout))
            .credentials(credentials)
    };

    Ok(builder.build())
}

fn sanitize_smtp_error(raw: &str, password: &str) -> String {
    let mut message = raw.to_string();
    if !password.is_empty() {
        message = message.replace(password, "***");
    }
    format!("mail.smtp_send_failed: {message}")
}
