//! Outbound mail sending use case.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use common::contracts::{MailIpcSendRequest, MailIpcSendResponse};
use ports::customer::CustomerStore;
use ports::mail::{MailSendInput, MailStore};

/// Record and mark one outbound message as sent.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub struct SendMail;

impl SendMail {
    /// Persist one outbound message after validating referenced customer/account/template ids.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    pub fn execute<M: MailStore + ?Sized, C: CustomerStore + ?Sized>(
        mail_store: &M,
        customer_store: &C,
        request: MailIpcSendRequest,
    ) -> Result<MailIpcSendResponse, String> {
        customer_store
            .get(&request.customer_id)
            .map_err(|error| error.to_string())?;
        mail_store
            .get_account(&request.account_id)
            .map_err(|error| error.to_string())?;
        mail_store
            .get_template(&request.template_id)
            .map_err(|error| error.to_string())?;

        let record = mail_store
            .create_outbound_message(MailSendInput {
                customer_id: request.customer_id,
                template_id: request.template_id,
                account_id: request.account_id,
                subject: request.subject,
                body_text: request.body_text,
                body_html: request.body_html,
                status: "sent".to_string(),
                sent_at: Some(now_string()),
                rfc_message_id: Some(format!("<{}@opendesk.local>", uuid::Uuid::new_v4())),
            })
            .map_err(|error| error.to_string())?;

        Ok(MailIpcSendResponse {
            message_id: record.id,
            status: record.status,
        })
    }
}

fn now_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{millis}")
}
