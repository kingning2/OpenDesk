use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeIpcToolDownloadResponse {
    pub ok: bool,
    pub error_message: Option<String>,
}
