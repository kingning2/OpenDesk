//! Outbound mail sending use case.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use common::contracts::{MailIpcSendRequest, MailIpcSendResponse};
use mail_net::{send_smtp, SmtpEndpoint, SmtpOutboundMessage};
use ports::customer::CustomerStore;
use ports::mail::{MailSendInput, MailStore};

/// Send one outbound message via SMTP and persist the result.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub struct SendMail;

impl SendMail {
    /// Validate refs, deliver over SMTP, then store sent/failed status.
    ///
    /// Recipient comes from `to_address`. Optional `customer_id` only links CRM.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    ///
    /// # 参数
    ///
    /// * `mail_store` - Mail account / template / message store
    /// * `customer_store` - Customer lookup when `customer_id` is set
    /// * `request` - Compose payload from UI
    ///
    /// # 返回值
    ///
    /// Stored `message_id` and final `sent` / `failed` status.
    pub fn execute<M: MailStore + ?Sized, C: CustomerStore + ?Sized>(
        mail_store: &M,
        customer_store: &C,
        request: MailIpcSendRequest,
    ) -> Result<MailIpcSendResponse, String> {
        let to_address = request.to_address.trim().to_string();
        if to_address.is_empty() || !to_address.contains('@') {
            return Err("mail.send.to_address_invalid".to_string());
        }

        let customer_id = request
            .customer_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        if let Some(ref id) = customer_id {
            customer_store.get(id).map_err(|error| error.to_string())?;
        }

        let account = mail_store
            .get_account(&request.account_id)
            .map_err(|error| error.to_string())?;

        let template_id = request
            .template_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let (template_id, template_name) = if let Some(id) = template_id {
            let template = mail_store
                .get_template(&id)
                .map_err(|error| error.to_string())?;
            (Some(template.id), template.name)
        } else {
            (None, "freeform".to_string())
        };

        let password = mail_store
            .resolve_account_password(&request.account_id)
            .map_err(|error| error.to_string())?;

        let tracking_enabled = request.open_tracking_enabled.unwrap_or(true);
        let tracking_id = if tracking_enabled {
            Some(crate::app::tracking::make_tracking_id())
        } else {
            None
        };
        let body_html = if let Some(ref tracking_id) = tracking_id {
            crate::app::tracking::prepare_tracked_html(
                &request.body_text,
                request.body_html.as_deref(),
                tracking_id,
                &to_address,
            )
        } else {
            crate::app::tracking::prepare_outbound_html(
                &request.body_text,
                request.body_html.as_deref(),
            )
        };

        let port = u16::try_from(account.smtp_port).map_err(|_| "mail.smtp_port_invalid")?;
        let smtp_result = send_smtp(
            &SmtpEndpoint {
                host: account.smtp_host.clone(),
                port,
                use_tls: account.use_tls,
                username: account.username.clone(),
                password,
            },
            &SmtpOutboundMessage {
                from_address: account.from_address.clone(),
                from_name: account.from_name.clone(),
                to_address: to_address.clone(),
                subject: request.subject.clone(),
                body_text: request.body_text.clone(),
                body_html,
            },
        );

        let (status, error_message, sent_at, rfc_message_id) = match smtp_result {
            Ok(_) => (
                "sent".to_string(),
                None,
                Some(now_string()),
                Some(format!("<{}@opendesk.local>", uuid::Uuid::new_v4())),
            ),
            Err(error) => ("failed".to_string(), Some(error), None, None),
        };

        let open_tracking_id = if status == "sent" { tracking_id } else { None };

        let record = mail_store
            .create_outbound_message(MailSendInput {
                customer_id,
                to_address,
                template_id,
                template_name,
                account_id: request.account_id,
                subject: request.subject,
                body_text: request.body_text,
                body_html: request.body_html,
                status,
                error_message,
                sent_at,
                rfc_message_id,
                from_address: account.from_address.clone(),
                open_tracking_id,
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
