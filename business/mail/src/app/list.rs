//! Local mailbox message list use case.
//!
//! 作者：coisini
//! 创建时间：2026-07-22

use common::contracts::{MailIpcMessageListRequest, MailIpcMessageListResponse};
use ports::mail::{MailMessageListFilter, MailStore};

use super::mapper::messages_to_json;
use super::tracking::fetch_open_status;

/// List local inbox/sent messages for the mail workbench.
///
/// 作者：coisini
/// 创建时间：2026-07-22
pub struct ListMailMessages;

impl ListMailMessages {
    /// Load paged local messages by direction and optional filters.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-22
    ///
    /// # 参数
    ///
    /// * `store` - Mail message store
    /// * `request` - List filters from UI
    ///
    /// # 返回值
    ///
    /// JSON-wrapped message list and total count.
    pub fn execute<S: MailStore + ?Sized>(
        store: &S,
        request: MailIpcMessageListRequest,
    ) -> Result<MailIpcMessageListResponse, String> {
        let (mut messages, total) = store
            .list_messages(MailMessageListFilter {
                direction: request.direction.clone(),
                account_id: request.account_id,
                customer_id: request.customer_id,
                query: request.query,
                limit: request.limit.unwrap_or(100),
                offset: request.offset.unwrap_or(0),
            })
            .map_err(|error| error.to_string())?;

        if request.sync_open_status.unwrap_or(false)
            && request.direction.eq_ignore_ascii_case("outbound")
        {
            let integration = store
                .get_email_read_integration()
                .map_err(|error| error.to_string())?;
            sync_outbound_open_status(store, &integration, &mut messages);
        }

        Ok(MailIpcMessageListResponse {
            messages_json: messages_to_json(&messages)?,
            total,
        })
    }
}

fn sync_outbound_open_status<S: MailStore + ?Sized>(
    store: &S,
    integration: &ports::mail::MailEmailReadIntegrationConfig,
    messages: &mut [ports::mail::MailMessageRecord],
) {
    const MAX_SYNC_PER_REQUEST: usize = 10;
    let mut attempts = 0usize;

    for message in messages.iter_mut() {
        if attempts >= MAX_SYNC_PER_REQUEST {
            break;
        }
        if message.direction != "outbound" || message.status != "sent" {
            continue;
        }
        let tracking_id = message
            .open_tracking_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let recipient = message
            .to_address
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let (Some(tracking_id), Some(recipient)) = (tracking_id, recipient) else {
            continue;
        };
        if message.open_count > 0 {
            continue;
        }

        attempts += 1;

        let Some((opened_at, open_count)) = fetch_open_status(integration, recipient, tracking_id)
        else {
            continue;
        };
        if open_count <= 0 {
            continue;
        }

        if store
            .update_message_open_status(&message.id, opened_at.clone(), open_count)
            .is_ok()
        {
            message.opened_at = opened_at;
            message.open_count = open_count;
        }
    }
}
