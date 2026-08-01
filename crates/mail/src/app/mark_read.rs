//! Mark one local mail message as read.
//!
//! 作者：coisini
//! 创建时间：2026-08-01

use common::contracts::{MailIpcMessageMarkReadRequest, MailIpcMessageMarkReadResponse};
use ports::mail::MailStore;

/// Mark one message read in local storage.
///
/// 作者：coisini
/// 创建时间：2026-08-01
pub struct MarkMailMessageRead;

impl MarkMailMessageRead {
    /// Persist read state for the selected inbox message.
    ///
    /// 作者：coisini
    /// 创建时间：2026-08-01
    pub fn execute<S: MailStore + ?Sized>(
        store: &S,
        request: MailIpcMessageMarkReadRequest,
    ) -> Result<MailIpcMessageMarkReadResponse, String> {
        let record = store
            .mark_message_read(&request.message_id)
            .map_err(|error| error.to_string())?;

        Ok(MailIpcMessageMarkReadResponse {
            message_id: record.id,
            is_read: record.is_read,
        })
    }
}
