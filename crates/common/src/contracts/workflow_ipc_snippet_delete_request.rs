use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowIpcSnippetDeleteRequest {
    pub id: String,
}
