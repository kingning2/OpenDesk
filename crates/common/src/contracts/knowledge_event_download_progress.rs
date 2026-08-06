use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEventDownloadProgress {
    pub tool: String,
    pub bytes_downloaded: i64,
    pub bytes_total: i64,
    pub status: String,
    pub error_message: Option<String>,
}
