use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcGenerateHtmlRequest {
    pub text: String,
    pub trace_id: Option<String>,
}
