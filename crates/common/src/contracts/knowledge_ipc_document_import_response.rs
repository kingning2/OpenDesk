use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeIpcDocumentImportResponse {
    pub ok: bool,
    pub job_id: Option<String>,
    pub error_message: Option<String>,
}
