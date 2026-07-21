use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerIpcListResponse {
    pub ok: bool,
    pub customers_json: String,
    pub total: i64,
    pub trace_id: Option<String>,
}
