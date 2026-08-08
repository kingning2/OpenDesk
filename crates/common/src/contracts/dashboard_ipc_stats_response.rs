use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardIpcStatsResponse {
    pub ok: bool,
    pub stats_json: String,
    pub trace_id: Option<String>,
}
