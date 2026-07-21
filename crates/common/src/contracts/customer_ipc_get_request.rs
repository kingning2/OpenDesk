use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerIpcGetRequest {
    pub trace_id: Option<String>,
    pub id: String,
}
