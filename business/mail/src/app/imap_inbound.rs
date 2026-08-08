//! Persist IMAP inbound messages from known senders (outbound recipients or customers).
//!
//! 作者：coisini
//! 创建时间：2026-08-01

use mail_net::ImapFetchedMessage;
use ports::customer::CustomerStore;
use ports::mail::{MailImapInboundWriteInput, MailStore};

/// Max IMAP messages to FETCH per sync (most recent UIDs when backlog is large).
pub const IMAP_SYNC_MAX_FETCH: usize = 50;

/// Advance `last_uid` only to the highest UID actually FETCHed this pass.
///
/// 作者：coisini
/// 创建时间：2026-08-01
pub fn next_imap_sync_cursor(last_uid: i64, fetched_max_uid: u32) -> i64 {
    let fetched = i64::from(fetched_max_uid);
    if fetched > last_uid {
        fetched
    } else {
        last_uid
    }
}

/// Summary of one IMAP persist pass.
///
/// 作者：coisini
/// 创建时间：2026-08-01
#[derive(Debug, Clone, Copy, Default)]
pub struct PersistImapInboundSummary {
    /// Rows inserted into local inbox.
    pub inserted: usize,
    /// Skipped because the sender is not a prior outbound recipient or customer.
    pub skipped_unmatched: usize,
}

enum InboundMatchReason {
    PriorOutboundRecipient,
    Customer,
}

/// Whether an inbound sender should be persisted for this account.
///
/// 发件人满足其一即入库：
/// - 当前账号曾向其发送过 outbound 邮件
/// - 发件人邮箱在客户库中
///
/// 作者：coisini
/// 创建时间：2026-08-01
pub fn should_persist_inbound<M: MailStore + ?Sized, C: CustomerStore + ?Sized>(
    mail_store: &M,
    customer_store: &C,
    account_id: &str,
    from_address: &str,
) -> Result<bool, String> {
    Ok(evaluate_inbound_match(mail_store, customer_store, account_id, from_address)?.is_some())
}

fn evaluate_inbound_match<M: MailStore + ?Sized, C: CustomerStore + ?Sized>(
    mail_store: &M,
    customer_store: &C,
    account_id: &str,
    from_address: &str,
) -> Result<Option<InboundMatchReason>, String> {
    if mail_store
        .has_outbound_to_address(account_id, from_address)
        .map_err(|error| error.to_string())?
    {
        return Ok(Some(InboundMatchReason::PriorOutboundRecipient));
    }

    if customer_store
        .find_by_email(from_address)
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Ok(Some(InboundMatchReason::Customer));
    }

    Ok(None)
}

/// Insert IMAP messages from known outbound recipients or customers.
///
/// 作者：coisini
/// 创建时间：2026-08-01
pub fn persist_imap_fetched_messages<M: MailStore + ?Sized, C: CustomerStore + ?Sized>(
    mail_store: &M,
    customer_store: &C,
    account_id: &str,
    folder: &str,
    messages: impl IntoIterator<Item = ImapFetchedMessage>,
) -> Result<PersistImapInboundSummary, String> {
    let mut summary = PersistImapInboundSummary::default();
    let messages: Vec<ImapFetchedMessage> = messages.into_iter().collect();

    if messages.is_empty() {
        tracing::info!(
            target: "lifecycle",
            %account_id,
            folder,
            "imap inbound persist: no messages"
        );
        return Ok(summary);
    }

    tracing::info!(
        target: "lifecycle",
        %account_id,
        folder,
        count = messages.len(),
        uids = ?messages.iter().map(|message| message.uid).collect::<Vec<_>>(),
        "imap inbound persist started"
    );

    for message in messages {
        let prior_outbound = mail_store
            .has_outbound_to_address(account_id, &message.from_address)
            .map_err(|error| error.to_string())?;
        let customer = customer_store
            .find_by_email(&message.from_address)
            .map_err(|error| error.to_string())?;
        let is_customer = customer.is_some();

        log_inbound_message_snapshot(account_id, folder, &message, prior_outbound, is_customer);

        let match_reason = evaluate_inbound_match(
            mail_store,
            customer_store,
            account_id,
            &message.from_address,
        )?;

        let Some(match_reason) = match_reason else {
            summary.skipped_unmatched += 1;
            tracing::info!(
                target: "lifecycle",
                %account_id,
                folder,
                uid = message.uid,
                from = %message.from_address,
                prior_outbound,
                is_customer,
                decision = "skip",
                reason = "sender not outbound recipient or customer",
                "imap inbound decision"
            );
            continue;
        };

        let customer_id = customer.map(|record| record.id);
        let match_label = match match_reason {
            InboundMatchReason::PriorOutboundRecipient => "prior_outbound_recipient",
            InboundMatchReason::Customer => "customer",
        };

        match mail_store
            .insert_imap_inbound_if_new(MailImapInboundWriteInput {
                account_id: account_id.to_string(),
                customer_id,
                from_address: message.from_address.clone(),
                from_name: message.from_name.clone(),
                subject: message.subject.clone(),
                body_text: message.body_text,
                body_html: message.body_html,
                received_at: message.received_at,
                imap_uid: message.uid as i64,
                imap_folder: folder.to_string(),
                rfc_message_id: message.rfc_message_id.clone(),
                in_reply_to: message.in_reply_to.clone(),
                references: message.references.clone(),
                is_seen: message.is_seen,
            })
            .map_err(|error| error.to_string())?
        {
            Some(record) => {
                summary.inserted += 1;
                tracing::info!(
                    target: "lifecycle",
                    %account_id,
                    folder,
                    uid = message.uid,
                    local_id = %record.id,
                    from = %message.from_address,
                    match_reason = match_label,
                    decision = "insert",
                    "imap inbound decision"
                );
            }
            None => {
                tracing::info!(
                    target: "lifecycle",
                    %account_id,
                    folder,
                    uid = message.uid,
                    from = %message.from_address,
                    match_reason = match_label,
                    decision = "skip",
                    reason = "duplicate rfc_message_id or imap uid",
                    "imap inbound decision"
                );
            }
        }
    }

    tracing::info!(
        target: "lifecycle",
        %account_id,
        folder,
        inserted = summary.inserted,
        skipped_unmatched = summary.skipped_unmatched,
        "imap inbound persist finished"
    );

    Ok(summary)
}

fn log_inbound_message_snapshot(
    account_id: &str,
    folder: &str,
    message: &ImapFetchedMessage,
    prior_outbound: bool,
    is_customer: bool,
) {
    let body_html_len = message
        .body_html
        .as_ref()
        .map(|html| html.len())
        .unwrap_or(0);
    tracing::info!(
        target: "lifecycle",
        %account_id,
        folder,
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
        prior_outbound,
        is_customer,
        body_text_len = message.body_text.len(),
        body_html_len,
        body_text_preview = %log_body_preview(&message.body_text),
        body_html_preview = %message
            .body_html
            .as_deref()
            .map(log_body_preview)
            .unwrap_or_else(|| "-".to_string()),
        raw_headers = %log_body_preview(&message.raw_headers),
        "imap inbound evaluating message"
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
