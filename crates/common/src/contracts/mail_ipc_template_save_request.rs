use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcTemplateSaveRequest {
    pub id: Option<String>,
    pub name: String,
    pub template_intent: String,
    pub subject_template: String,
    pub body_text_template: String,
    pub body_html_template: Option<String>,
    pub locale: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i64>,
}
