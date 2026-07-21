use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerIpcUpdateResponse {
    pub ok: bool,
    pub profile_json: String,
    pub trace_id: Option<String>,
}
