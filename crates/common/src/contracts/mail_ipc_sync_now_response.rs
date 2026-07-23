use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailIpcSyncNowResponse {
    pub job_ids_json: String,
    pub enqueued: i64,
}
