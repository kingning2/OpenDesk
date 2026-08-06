use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeIpcDocumentDeleteResponse {
    pub ok: bool,
}
