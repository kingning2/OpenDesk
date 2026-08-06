use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeIpcToolDownloadRequest {
    pub tool: String,
}
