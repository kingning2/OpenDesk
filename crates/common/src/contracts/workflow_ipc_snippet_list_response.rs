use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowIpcSnippetListResponse {
    pub snippets_json: String,
    pub total: i64,
}
