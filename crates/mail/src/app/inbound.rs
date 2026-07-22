//! Manual inbound record use case.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use common::contracts::{MailIpcRecordInboundRequest, MailIpcRecordInboundResponse};
use ports::customer::CustomerStore;
use ports::mail::{MailInboundWriteInput, MailStore};

/// Record one inbound message manually.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub struct RecordInboundMail;

impl RecordInboundMail {
    /// Persist one inbound message and append a customer timeline entry.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    pub fn execute<M: MailStore + ?Sized, C: CustomerStore + ?Sized>(
        mail_store: &M,
        customer_store: &C,
        request: MailIpcRecordInboundRequest,
    ) -> Result<MailIpcRecordInboundResponse, String> {
        customer_store
            .get(&request.customer_id)
            .map_err(|error| error.to_string())?;

        let record = mail_store
            .create_inbound_message(MailInboundWriteInput {
                customer_id: request.customer_id,
                from_address: request.from_address,
                from_name: request.from_name,
                subject: request.subject,
                body_text: request.body_text,
                body_html: request.body_html,
                received_at: request.received_at,
                rfc_message_id: request.rfc_message_id,
                in_reply_to: request.in_reply_to,
            })
            .map_err(|error| error.to_string())?;

        Ok(MailIpcRecordInboundResponse {
            message_id: record.id,
        })
    }
}
