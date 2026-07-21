use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerEventChannelAccepted {
    pub event_id: String,
    pub occurred_at: String,
    pub job_id: String,
    pub platform: String,
    pub keyword: String,
    pub channel_id: String,
    pub title: String,
    pub country: Option<String>,
    pub subscriber_count: Option<i64>,
    pub email: Option<String>,
    pub description: Option<String>,
    pub custom_url: Option<String>,
    pub email_status: String,
    pub enrich_attempts: Option<i64>,
    pub enrich_error: Option<String>,
    pub enriched_at: Option<String>,
}
