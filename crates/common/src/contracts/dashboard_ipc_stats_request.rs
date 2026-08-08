use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardIpcStatsRequest {
    pub trace_id: Option<String>,
}
