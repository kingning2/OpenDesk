use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEventImportUpdated {
    pub document_id: String,
    pub status: String,
    pub error_message: Option<String>,
}
