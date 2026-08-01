use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcGenerateHtmlResponse {
    pub ok: bool,
    pub html: String,
    pub notes: Option<String>,
    pub message: String,
    pub trace_id: Option<String>,
}
