use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeIpcToolStatusResponse {
    pub tools_json: String,
}
