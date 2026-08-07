use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDtoWorkflowBinding {
    pub account_id: String,
    pub template_id: String,
}
