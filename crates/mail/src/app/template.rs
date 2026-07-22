//! Mail template listing and rendering use cases.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use common::contracts::{
    MailIpcTemplateApplyRequest, MailIpcTemplateApplyResponse, MailIpcTemplateListResponse,
};
use ports::customer::CustomerStore;
use ports::mail::MailStore;

use super::mapper::templates_to_json;
use crate::domain::DEFAULT_SENDER_NAME;

/// List available mail templates.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub struct ListMailTemplates;

impl ListMailTemplates {
    /// Load all templates from the mail store.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    ///
    /// # 参数
    ///
    /// * `store` - Mail template store
    ///
    /// # 返回值
    ///
    /// JSON-wrapped template list response.
    pub fn execute<S: MailStore + ?Sized>(
        store: &S,
    ) -> Result<MailIpcTemplateListResponse, String> {
        let templates = store.list_templates().map_err(|error| error.to_string())?;
        Ok(MailIpcTemplateListResponse {
            templates_json: templates_to_json(&templates)?,
            total: templates.len() as i64,
        })
    }
}

/// Render a template for one customer.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub struct ApplyMailTemplate;

impl ApplyMailTemplate {
    /// Apply one template to one customer and optional sender account.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    ///
    /// # 参数
    ///
    /// * `mail_store` - Mail store for templates and optional account
    /// * `customer_store` - Customer store for template variables
    /// * `request` - Template apply request
    ///
    /// # 返回值
    ///
    /// Rendered subject and body preview.
    pub fn execute<M: MailStore + ?Sized, C: CustomerStore + ?Sized>(
        mail_store: &M,
        customer_store: &C,
        request: MailIpcTemplateApplyRequest,
    ) -> Result<MailIpcTemplateApplyResponse, String> {
        let template = mail_store
            .get_template(&request.template_id)
            .map_err(|error| error.to_string())?;
        let customer = customer_store
            .get(&request.customer_id)
            .map_err(|error| error.to_string())?;

        let sender = request
            .account_id
            .as_deref()
            .and_then(|id| mail_store.get_account(id).ok());
        let sender_name = sender
            .as_ref()
            .and_then(|account| account.from_name.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_SENDER_NAME.to_string());
        let sender_email = sender
            .as_ref()
            .map(|account| account.from_address.clone())
            .unwrap_or_default();

        Ok(MailIpcTemplateApplyResponse {
            subject: render_text(
                &template.subject_template,
                &customer,
                &sender_name,
                &sender_email,
            ),
            body_text: render_text(
                &template.body_text_template,
                &customer,
                &sender_name,
                &sender_email,
            ),
            body_html: template
                .body_html_template
                .map(|value| render_text(&value, &customer, &sender_name, &sender_email)),
        })
    }
}

fn render_text(
    raw: &str,
    customer: &ports::customer::CustomerRecord,
    sender_name: &str,
    sender_email: &str,
) -> String {
    raw.replace(
        "{{customer.display_name}}",
        customer.display_name.as_deref().unwrap_or(&customer.email),
    )
    .replace("{{customer.email}}", &customer.email)
    .replace(
        "{{customer.source_title}}",
        customer.source_meta.as_deref().unwrap_or(""),
    )
    .replace(
        "{{customer.quoted_price}}",
        &customer
            .quoted_price
            .map(|value| format!("{value}"))
            .unwrap_or_default(),
    )
    .replace(
        "{{customer.currency}}",
        customer.quoted_currency.as_deref().unwrap_or(""),
    )
    .replace(
        "{{customer.package_name}}",
        customer.package_name.as_deref().unwrap_or(""),
    )
    .replace(
        "{{customer.pricing_tier}}",
        customer.pricing_tier.as_deref().unwrap_or(""),
    )
    .replace("{{sender.name}}", sender_name)
    .replace("{{sender.email}}", sender_email)
    .replace("{{today.date}}", "today")
}
