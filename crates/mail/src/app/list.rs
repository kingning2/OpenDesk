//! Local mailbox message list use case.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-22

use common::contracts::{MailIpcMessageListRequest, MailIpcMessageListResponse};
use ports::mail::{MailMessageListFilter, MailStore};

use super::mapper::messages_to_json;

/// List local inbox/sent messages for the mail workbench.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub struct ListMailMessages;

impl ListMailMessages {
    /// Load paged local messages by direction and optional filters.
    ///
    /// 作者：Xiaoman
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
        let (messages, total) = store
            .list_messages(MailMessageListFilter {
                direction: request.direction,
                account_id: request.account_id,
                customer_id: request.customer_id,
                query: request.query,
                limit: request.limit.unwrap_or(100),
                offset: request.offset.unwrap_or(0),
            })
            .map_err(|error| error.to_string())?;

        Ok(MailIpcMessageListResponse {
            messages_json: messages_to_json(&messages)?,
            total,
        })
    }
}
