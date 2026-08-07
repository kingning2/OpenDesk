use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowIpcScriptListRequest {
    pub category_l1: Option<String>,
    pub category_l2: Option<String>,
    pub query: Option<String>,
}
