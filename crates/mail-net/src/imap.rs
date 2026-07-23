//! IMAP transport for inbound business mail.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-22

use std::net::TcpStream;
use std::time::Duration;

use imap::Session;
use mailparse::{parse_mail, MailHeaderMap};
use native_tls::TlsConnector;

/// IMAP connection settings for one inbox sync.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[derive(Debug, Clone)]
pub struct ImapEndpoint {
    /// IMAP host name.
    pub host: String,
    /// IMAP port (993 implicit TLS / 143 plain).
    pub port: u16,
    /// Whether TLS should be used.
    pub use_tls: bool,
    /// IMAP auth username.
    pub username: String,
    /// IMAP auth password (never log).
    pub password: String,
}

/// One message fetched from IMAP INBOX.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[derive(Debug, Clone)]
pub struct ImapFetchedMessage {
    /// IMAP UID within the selected folder.
    pub uid: u32,
    /// RFC Message-ID header when present.
    pub rfc_message_id: Option<String>,
    /// Sender email address.
    pub from_address: String,
    /// Optional sender display name.
    pub from_name: Option<String>,
    /// Subject line.
    pub subject: String,
    /// Plain-text body.
    pub body_text: String,
    /// Optional HTML body.
    pub body_html: Option<String>,
    /// Received timestamp (ISO-8601 UTC).
    pub received_at: String,
    /// In-Reply-To header when present.
    pub in_reply_to: Option<String>,
    /// References header when present.
    pub references: Option<String>,
}

/// Fetch messages with UID greater than `last_uid` from one folder.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
///
/// # 参数
///
/// * `endpoint` - IMAP host / auth settings
/// * `folder` - Mailbox folder (MVP: `INBOX`)
/// * `last_uid` - Last persisted UID; fetch `last_uid + 1:*`
///
/// # 返回值
///
/// * `Ok(messages)` - New messages in ascending UID order
/// * `Err(message)` - Connection, auth, or parse failure (no password included)
pub fn fetch_messages_since(
    endpoint: &ImapEndpoint,
    folder: &str,
    last_uid: u32,
) -> Result<Vec<ImapFetchedMessage>, String> {
    let mut session = connect(endpoint)?;
    session
        .select(folder)
        .map_err(|error| format!("imap.select_failed: {error}"))?;

    let query = if last_uid == 0 {
        "ALL".to_string()
    } else {
        format!("{}:*", last_uid.saturating_add(1))
    };

    let uid_set = session
        .uid_search(&query)
        .map_err(|error| format!("imap.search_failed: {error}"))?;

    let mut uids: Vec<u32> = uid_set.into_iter().filter(|uid| *uid > last_uid).collect();
    uids.sort_unstable();

    let mut messages = Vec::new();
    for uid in uids {
        let fetched = session
            .uid_fetch(uid.to_string(), "RFC822")
            .map_err(|error| format!("imap.fetch_failed: {error}"))?;
        let Some(body) = fetched.iter().find_map(|item| item.body()) else {
            continue;
        };
        if let Ok(parsed) = parse_fetched(uid, body) {
            messages.push(parsed);
        }
    }

    let _ = session.logout();
    Ok(messages)
}

/// Watch one mailbox with IMAP IDLE and fetch new messages.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
///
/// # 参数
///
/// * `endpoint` - IMAP host / auth settings
/// * `folder` - Mailbox folder (MVP: `INBOX`)
/// * `last_uid` - Last persisted UID; initial catch-up fetches after this UID
/// * `on_messages` - Called whenever new messages are fetched
pub fn watch_inbox_idle<F>(
    endpoint: &ImapEndpoint,
    folder: &str,
    last_uid: u32,
    mut on_messages: F,
) -> Result<(), String>
where
    F: FnMut(Vec<ImapFetchedMessage>) -> Result<u32, String>,
{
    let mut session = connect(endpoint)?;
    session
        .select(folder)
        .map_err(|error| format!("imap.select_failed: {error}"))?;

    let mut cursor = last_uid;
    let initial = fetch_new_messages(&mut session, cursor)?;
    if !initial.is_empty() {
        cursor = on_messages(initial)?;
    }

    loop {
        let mut idle_handle = session
            .idle()
            .map_err(|error| format!("imap.idle_init_failed: {error}"))?;
        idle_handle.set_keepalive(Duration::from_secs(60));
        idle_handle
            .wait_keepalive()
            .map_err(|error| format!("imap.idle_wait_failed: {error}"))?;

        let next = fetch_new_messages(&mut session, cursor)?;
        if next.is_empty() {
            continue;
        }
        cursor = on_messages(next)?;
    }
}

fn connect(endpoint: &ImapEndpoint) -> Result<Session<native_tls::TlsStream<TcpStream>>, String> {
    if !endpoint.use_tls {
        return Err("imap.tls_required".to_string());
    }

    let tls = TlsConnector::builder()
        .build()
        .map_err(|error| format!("imap.tls_init: {error}"))?;

    let client = imap::connect(
        (endpoint.host.as_str(), endpoint.port),
        endpoint.host.as_str(),
        &tls,
    )
    .map_err(|error| sanitize_imap_error(&error.to_string(), &endpoint.password))?;

    client
        .login(&endpoint.username, &endpoint.password)
        .map_err(|error| sanitize_imap_error(&error.0.to_string(), &endpoint.password))
}

fn fetch_new_messages(
    session: &mut Session<native_tls::TlsStream<TcpStream>>,
    last_uid: u32,
) -> Result<Vec<ImapFetchedMessage>, String> {
    let query = if last_uid == 0 {
        "ALL".to_string()
    } else {
        format!("{}:*", last_uid.saturating_add(1))
    };

    let uid_set = session
        .uid_search(&query)
        .map_err(|error| format!("imap.search_failed: {error}"))?;

    let mut uids: Vec<u32> = uid_set.into_iter().filter(|uid| *uid > last_uid).collect();
    uids.sort_unstable();

    let mut messages = Vec::new();
    for uid in uids {
        let fetched = session
            .uid_fetch(uid.to_string(), "RFC822")
            .map_err(|error| format!("imap.fetch_failed: {error}"))?;
        let Some(body) = fetched.iter().find_map(|item| item.body()) else {
            continue;
        };
        if let Ok(parsed) = parse_fetched(uid, body) {
            messages.push(parsed);
        }
    }

    Ok(messages)
}

fn parse_fetched(uid: u32, body: &[u8]) -> Result<ImapFetchedMessage, String> {
    let parsed = parse_mail(body).map_err(|error| format!("imap.parse_failed: {error}"))?;
    let headers = parsed.get_headers();

    let from_raw = headers
        .get_first_value("From")
        .unwrap_or_else(|| "unknown@invalid".to_string());
    let (from_address, from_name) = parse_mailbox(&from_raw);

    let subject = headers
        .get_first_value("Subject")
        .unwrap_or_else(|| "(no subject)".to_string());

    let rfc_message_id = headers
        .get_first_value("Message-ID")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let in_reply_to = headers
        .get_first_value("In-Reply-To")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let references = headers
        .get_first_value("References")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let received_at = headers
        .get_first_value("Date")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(now_iso8601);

    let (body_text, body_html) = extract_bodies(&parsed);

    Ok(ImapFetchedMessage {
        uid,
        rfc_message_id,
        from_address,
        from_name,
        subject,
        body_text,
        body_html,
        received_at,
        in_reply_to,
        references,
    })
}

fn extract_bodies(parsed: &mailparse::ParsedMail<'_>) -> (String, Option<String>) {
    if parsed.subparts.is_empty() {
        let mime = parsed.ctype.mimetype.as_str();
        let body = parsed.get_body().unwrap_or_default();
        if mime == "text/html" {
            return (strip_html_tags(&body), Some(body));
        }
        return (body, None);
    }

    let mut text = None;
    let mut html = None;
    for part in &parsed.subparts {
        let (part_text, part_html) = extract_bodies(part);
        if text.is_none() && !part_text.trim().is_empty() {
            text = Some(part_text);
        }
        if html.is_none() {
            html = part_html;
        }
    }

    (
        text.unwrap_or_default(),
        html.filter(|value| !value.trim().is_empty()),
    )
}

fn parse_mailbox(raw: &str) -> (String, Option<String>) {
    let trimmed = raw.trim();
    if let Some((name, address)) = trimmed.split_once('<') {
        let address = address.trim_end_matches('>').trim().to_string();
        let name = name.trim().trim_matches('"').to_string();
        return (address, (!name.is_empty()).then_some(name));
    }
    (trimmed.to_string(), None)
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output
}

fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn sanitize_imap_error(raw: &str, password: &str) -> String {
    let mut message = raw.to_string();
    if !password.is_empty() {
        message = message.replace(password, "***");
    }
    format!("imap.error: {message}")
}
