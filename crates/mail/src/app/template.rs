//! Mail template listing, rendering, and custom save use cases.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use common::contracts::{
    MailIpcTemplateApplyRequest, MailIpcTemplateApplyResponse, MailIpcTemplateListResponse,
    MailIpcTemplateSaveRequest,
};
use ports::customer::CustomerStore;
use ports::mail::{MailStore, MailTemplateWriteInput};

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

/// Create or update a custom mail template.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub struct SaveMailTemplate;

impl SaveMailTemplate {
    /// Persist one custom template and return the refreshed list.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-22
    ///
    /// # 参数
    ///
    /// * `store` - Mail template store
    /// * `request` - Template save request
    ///
    /// # 返回值
    ///
    /// Updated template list response.
    pub fn execute<S: MailStore + ?Sized>(
        store: &S,
        request: MailIpcTemplateSaveRequest,
    ) -> Result<MailIpcTemplateListResponse, String> {
        let intent = request.template_intent.trim();
        if intent.is_empty() {
            return Err("mail.template.intent_required".to_string());
        }
        if request.name.trim().is_empty() {
            return Err("mail.template.name_required".to_string());
        }
        if request.subject_template.trim().is_empty()
            || request.body_text_template.trim().is_empty()
        {
            return Err("mail.template.content_required".to_string());
        }

        store
            .save_template(MailTemplateWriteInput {
                id: request.id,
                name: request.name.trim().to_string(),
                template_intent: intent.to_string(),
                subject_template: request.subject_template,
                body_text_template: request.body_text_template,
                body_html_template: request.body_html_template,
                locale: request.locale,
                is_active: request.is_active.unwrap_or(true),
                sort_order: request.sort_order.unwrap_or(100),
            })
            .map_err(|error| error.to_string())?;

        ListMailTemplates::execute(store)
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
    .replace("{{today.date}}", &today_date())
}

fn today_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    // UTC YYYY-MM-DD without extra chrono dependency.
    const DAY: u64 = 86_400;
    let days = secs / DAY;
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z.saturating_sub(era * 146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    format!("{year:04}-{m:02}-{d:02}")
}
