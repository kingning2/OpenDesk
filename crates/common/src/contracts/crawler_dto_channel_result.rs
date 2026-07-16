use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerDtoChannelResult {
    pub platform: String,
    pub channel_id: String,
    pub title: String,
    pub country: Option<String>,
    pub subscriber_count: Option<i64>,
    pub email: Option<String>,
    pub description: Option<String>,
    pub custom_url: Option<String>,
}
