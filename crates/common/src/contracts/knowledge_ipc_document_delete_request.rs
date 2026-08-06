use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeIpcDocumentDeleteRequest {
    pub document_id: String,
}
