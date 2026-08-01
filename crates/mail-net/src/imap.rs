//! IMAP transport for inbound business mail.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-22

use std::net::TcpStream;
use std::time::Duration;

use imap::Session;
use mailparse::{parse_mail, MailHeaderMap};
use native_tls::TlsConnector;

type ImapSession = Session<native_tls::TlsStream<TcpStream>>;

/// Parenthesized multi-attribute lists are required by Gmail/Feishu; bare `FLAGS BODY.PEEK[]` is invalid.
const FETCH_ATTR_CANDIDATES: &[&str] = &[
    "(FLAGS BODY.PEEK[])",
    "BODY.PEEK[]",
    "(FLAGS RFC822)",
    "RFC822",
];
const FETCH_BATCH_SIZE: usize = 50;

/// Default cap for one IMAP sync when the mailbox backlog is large.
pub const DEFAULT_IMAP_MAX_FETCH: usize = 50;

/// Result of one IMAP UID search + fetch pass.
///
/// 作者：Xiaoman
/// 创建时间：2026-08-01
#[derive(Debug, Clone)]
pub struct ImapFetchResult {
    /// Parsed messages from FETCH (may be capped to the most recent UIDs).
    pub messages: Vec<ImapFetchedMessage>,
    /// Highest UID actually FETCHed in this pass — use this to advance `last_uid`.
    pub fetched_max_uid: u32,
    /// Highest UID in the search window (diagnostics).
    pub search_max_uid: u32,
}

struct UidSearchPlan {
    to_fetch: Vec<u32>,
    fetched_max_uid: u32,
    search_max_uid: u32,
    pending_total: usize,
    search_from_uid: u32,
}

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
    /// Whether IMAP `\Seen` was set on the server.
    pub is_seen: bool,
    /// Raw `To` header value.
    pub to_raw: Option<String>,
    /// Raw `Cc` header value.
    pub cc_raw: Option<String>,
    /// Raw RFC822 header block (for diagnostics; not persisted).
    pub raw_headers: String,
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
/// * `max_fetch` - Max messages to FETCH; when backlog is larger, only the most recent UIDs are fetched
///
/// # 返回值
///
/// * `Ok(result)` - Fetched messages and search cursor UID
/// * `Err(message)` - Connection, auth, or parse failure (no password included)
pub fn fetch_messages_since(
    endpoint: &ImapEndpoint,
    folder: &str,
    last_uid: u32,
    max_fetch: usize,
) -> Result<ImapFetchResult, String> {
    tracing::info!(
        target: "lifecycle",
        host = %endpoint.host,
        port = endpoint.port,
        folder,
        last_uid,
        user = %endpoint.username,
        "imap fetch started"
    );

    let mut session = connect(endpoint)?;
    session.select(folder).map_err(|error| {
        let message = format!("imap.select_failed: {error}");
        tracing::warn!(
            target: "lifecycle",
            host = %endpoint.host,
            folder,
            error = %message,
            "imap select failed"
        );
        message
    })?;

    let plan = search_uids_since(&mut session, last_uid, max_fetch)?;
    log_uid_search_plan(&endpoint.host, folder, &plan, max_fetch);

    let messages = uid_fetch_messages(&mut session, &plan.to_fetch)?;
    log_fetched_messages(folder, &messages, "poll");

    tracing::info!(
        target: "lifecycle",
        host = %endpoint.host,
        folder,
        fetched = messages.len(),
        "imap fetch completed"
    );

    let _ = session.logout();
    Ok(ImapFetchResult {
        messages,
        fetched_max_uid: plan.fetched_max_uid,
        search_max_uid: plan.search_max_uid,
    })
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
    max_fetch: usize,
    mut on_messages: F,
) -> Result<(), String>
where
    F: FnMut(ImapFetchResult) -> Result<u32, String>,
{
    tracing::info!(
        target: "lifecycle",
        host = %endpoint.host,
        port = endpoint.port,
        folder,
        last_uid,
        user = %endpoint.username,
        "imap idle session starting"
    );

    let mut session = connect(endpoint)?;
    session.select(folder).map_err(|error| {
        let message = format!("imap.select_failed: {error}");
        tracing::warn!(
            target: "lifecycle",
            host = %endpoint.host,
            folder,
            error = %message,
            "imap idle select failed"
        );
        message
    })?;

    let mut cursor = last_uid;
    tracing::info!(
        target: "lifecycle",
        host = %endpoint.host,
        folder,
        last_uid = cursor,
        "imap idle initial catch-up"
    );
    let initial = fetch_new_messages(
        &mut session,
        &endpoint.host,
        folder,
        cursor,
        max_fetch,
        "initial",
    )?;
    if initial.fetched_max_uid > cursor || !initial.messages.is_empty() {
        cursor = on_messages(initial)?;
    }

    tracing::info!(
        target: "lifecycle",
        host = %endpoint.host,
        folder,
        cursor_uid = cursor,
        "imap idle listening"
    );

    loop {
        let mut idle_handle = session
            .idle()
            .map_err(|error| format!("imap.idle_init_failed: {error}"))?;
        idle_handle.set_keepalive(Duration::from_secs(60));
        idle_handle
            .wait_keepalive()
            .map_err(|error| format!("imap.idle_wait_failed: {error}"))?;

        tracing::info!(
            target: "lifecycle",
            host = %endpoint.host,
            folder,
            cursor_uid = cursor,
            "imap idle notified, fetching new mail"
        );

        let next = fetch_new_messages(
            &mut session,
            &endpoint.host,
            folder,
            cursor,
            max_fetch,
            "idle",
        )?;
        if next.fetched_max_uid <= cursor && next.messages.is_empty() {
            tracing::info!(
                target: "lifecycle",
                host = %endpoint.host,
                folder,
                cursor_uid = cursor,
                "imap idle fetch: no new messages"
            );
            continue;
        }
        cursor = on_messages(next)?;
    }
}

fn connect(endpoint: &ImapEndpoint) -> Result<ImapSession, String> {
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
    .map_err(|error| {
        let message = sanitize_imap_error(&error.to_string(), &endpoint.password);
        tracing::warn!(
            target: "lifecycle",
            host = %endpoint.host,
            port = endpoint.port,
            error = %message,
            "imap connect failed"
        );
        message
    })?;

    client
        .login(&endpoint.username, &endpoint.password)
        .map_err(|error| {
            let message = sanitize_imap_error(&error.0.to_string(), &endpoint.password);
            tracing::warn!(
                target: "lifecycle",
                host = %endpoint.host,
                user = %endpoint.username,
                error = %message,
                "imap login failed"
            );
            message
        })
}

fn fetch_new_messages(
    session: &mut ImapSession,
    host: &str,
    folder: &str,
    last_uid: u32,
    max_fetch: usize,
    reason: &str,
) -> Result<ImapFetchResult, String> {
    let plan = search_uids_since(session, last_uid, max_fetch)?;
    log_uid_search_plan(host, folder, &plan, max_fetch);
    let messages = uid_fetch_messages(session, &plan.to_fetch)?;
    log_fetched_messages(folder, &messages, reason);
    Ok(ImapFetchResult {
        messages,
        fetched_max_uid: plan.fetched_max_uid,
        search_max_uid: plan.search_max_uid,
    })
}

fn log_uid_search_plan(host: &str, folder: &str, plan: &UidSearchPlan, max_fetch: usize) {
    tracing::info!(
        target: "lifecycle",
        %host,
        folder,
        search_from_uid = plan.search_from_uid,
        pending_uids = plan.pending_total,
        fetch_uids = plan.to_fetch.len(),
        fetched_max_uid = plan.fetched_max_uid,
        search_max_uid = plan.search_max_uid,
        fetch_uid_list = ?plan.to_fetch,
        "imap uid search completed"
    );
    if plan.pending_total > plan.to_fetch.len() {
        tracing::info!(
            target: "lifecycle",
            %host,
            folder,
            pending_uids = plan.pending_total,
            max_fetch,
            search_max_uid = plan.search_max_uid,
            "imap fetch capped to recent uids"
        );
    }
}

fn log_fetched_messages(folder: &str, messages: &[ImapFetchedMessage], reason: &str) {
    if messages.is_empty() {
        tracing::info!(
            target: "lifecycle",
            folder,
            reason,
            count = 0,
            "imap fetch batch empty"
        );
        return;
    }

    tracing::info!(
        target: "lifecycle",
        folder,
        reason,
        count = messages.len(),
        uids = ?messages.iter().map(|message| message.uid).collect::<Vec<_>>(),
        "imap fetch batch"
    );

    for message in messages {
        log_fetched_message_detail(folder, reason, message);
    }
}

/// Log one parsed IMAP message with headers and body preview for debugging.
fn log_fetched_message_detail(folder: &str, reason: &str, message: &ImapFetchedMessage) {
    let body_html_len = message
        .body_html
        .as_ref()
        .map(|html| html.len())
        .unwrap_or(0);
    tracing::info!(
        target: "lifecycle",
        folder,
        reason,
        uid = message.uid,
        is_seen = message.is_seen,
        from_name = message.from_name.as_deref().unwrap_or("-"),
        from = %message.from_address,
        to = message.to_raw.as_deref().unwrap_or("-"),
        cc = message.cc_raw.as_deref().unwrap_or("-"),
        subject = %message.subject,
        date = %message.received_at,
        message_id = message.rfc_message_id.as_deref().unwrap_or("-"),
        in_reply_to = message.in_reply_to.as_deref().unwrap_or("-"),
        references = message.references.as_deref().unwrap_or("-"),
        body_text_len = message.body_text.len(),
        body_html_len,
        body_text_preview = %log_body_preview(&message.body_text),
        body_html_preview = %message
            .body_html
            .as_deref()
            .map(log_body_preview)
            .unwrap_or_else(|| "-".to_string()),
        raw_headers = %log_body_preview(&message.raw_headers),
        "imap raw message fetched"
    );
}

fn log_body_preview(body: &str) -> String {
    const MAX: usize = 500;
    let collapsed: String = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= MAX {
        collapsed
    } else {
        format!("{}…", collapsed.chars().take(MAX).collect::<String>())
    }
}

fn search_uids_since(
    session: &mut ImapSession,
    last_uid: u32,
    max_fetch: usize,
) -> Result<UidSearchPlan, String> {
    let max_fetch = max_fetch.max(1);
    let search_from_uid = if last_uid == 0 {
        0
    } else {
        last_uid.saturating_sub(max_fetch as u32)
    };
    let query = if search_from_uid == 0 {
        "ALL".to_string()
    } else {
        format!("{}:*", search_from_uid.max(1))
    };

    let uid_set = session.uid_search(&query).map_err(|error| {
        let message = format!("imap.search_failed: {error}");
        tracing::warn!(
            target: "lifecycle",
            query = %query,
            error = %message,
            "imap uid search failed"
        );
        message
    })?;

    let mut uids: Vec<u32> = uid_set
        .into_iter()
        .filter(|uid| *uid > search_from_uid)
        .collect();
    uids.sort_unstable();
    let pending_total = uids.len();
    let search_max_uid = uids.last().copied().unwrap_or(last_uid);
    let to_fetch = if pending_total > max_fetch {
        uids[pending_total - max_fetch..].to_vec()
    } else {
        uids
    };
    let fetched_max_uid = to_fetch.last().copied().unwrap_or(last_uid);
    Ok(UidSearchPlan {
        to_fetch,
        fetched_max_uid,
        search_max_uid,
        pending_total,
        search_from_uid,
    })
}

fn uid_fetch_messages(
    session: &mut ImapSession,
    uids: &[u32],
) -> Result<Vec<ImapFetchedMessage>, String> {
    let mut messages = Vec::with_capacity(uids.len());
    for chunk in uids.chunks(FETCH_BATCH_SIZE) {
        match uid_fetch_chunk(session, chunk) {
            Ok(batch) => messages.extend(batch),
            Err(_batch_error) if chunk.len() > 1 => {
                for uid in chunk {
                    messages.extend(uid_fetch_chunk(session, std::slice::from_ref(uid))?);
                }
            }
            Err(batch_error) => return Err(batch_error),
        }
    }
    Ok(messages)
}

fn uid_fetch_chunk(
    session: &mut ImapSession,
    uids: &[u32],
) -> Result<Vec<ImapFetchedMessage>, String> {
    let uid_set = uids
        .iter()
        .map(|uid| uid.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let mut last_error = String::new();
    for attrs in FETCH_ATTR_CANDIDATES {
        match session.uid_fetch(&uid_set, attrs) {
            Ok(fetched) => {
                if *attrs != FETCH_ATTR_CANDIDATES[0] {
                    tracing::info!(
                        target: "lifecycle",
                        uids = %uid_set,
                        attrs,
                        "imap fetch succeeded with fallback attrs"
                    );
                }
                let mut messages = Vec::new();
                for item in fetched.iter() {
                    if let Some(message) = parse_fetch_item(item) {
                        messages.push(message);
                    } else if let Some(uid) = item.uid {
                        let body_len = item.body().map(|body| body.len()).unwrap_or(0);
                        tracing::warn!(
                            target: "lifecycle",
                            uid,
                            body_len,
                            "imap fetch item parse failed"
                        );
                    }
                }
                return Ok(messages);
            }
            Err(error) => {
                last_error = format!("imap.fetch_failed: {error}");
                tracing::warn!(
                    target: "lifecycle",
                    uids = %uid_set,
                    attrs,
                    error = %last_error,
                    "imap uid fetch attempt failed"
                );
            }
        }
    }

    Err(last_error)
}

fn parse_fetch_item(item: &imap::types::Fetch) -> Option<ImapFetchedMessage> {
    let uid = item.uid?;
    let body = item.body()?;
    let is_seen = item
        .flags()
        .iter()
        .any(|flag| matches!(flag, imap::types::Flag::Seen));
    parse_fetched(uid, body, is_seen).ok()
}

fn parse_fetched(uid: u32, body: &[u8], is_seen: bool) -> Result<ImapFetchedMessage, String> {
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
    let to_raw = headers
        .get_first_value("To")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let cc_raw = headers
        .get_first_value("Cc")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let received_at = headers
        .get_first_value("Date")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(now_iso8601);

    let (body_text, body_html) = extract_bodies(&parsed);
    let raw_headers = extract_raw_headers(body);

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
        is_seen,
        to_raw,
        cc_raw,
        raw_headers,
    })
}

fn extract_raw_headers(body: &[u8]) -> String {
    const MAX: usize = 4_096;
    let text = String::from_utf8_lossy(body);
    let header_end = text
        .find("\r\n\r\n")
        .or_else(|| text.find("\n\n"))
        .unwrap_or(text.len());
    let headers = &text[..header_end.min(text.len())];
    if headers.chars().count() <= MAX {
        headers.to_string()
    } else {
        format!("{}…", headers.chars().take(MAX).collect::<String>())
    }
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
