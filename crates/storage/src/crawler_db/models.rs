//! Diesel row models for crawler tables.

use diesel::prelude::*;

use super::schema::{crawler_channel, crawler_keyword};

#[derive(Debug, Insertable)]
#[diesel(table_name = crawler_keyword)]
pub struct NewCrawlerKeyword {
    pub batch_id: String,
    pub text: String,
    pub enabled: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crawler_channel)]
pub struct NewCrawlerChannel {
    pub job_id: String,
    pub keyword: String,
    pub platform: String,
    pub channel_id: String,
    pub title: String,
    pub country: Option<String>,
    pub subscriber_count: Option<i64>,
    pub email: Option<String>,
    pub description: Option<String>,
    pub custom_url: Option<String>,
    pub email_status: String,
    pub enrich_attempts: i32,
    pub enrich_error: Option<String>,
    pub enriched_at: Option<String>,
    pub verified_email: Option<String>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crawler_channel)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CrawlerChannelRow {
    pub id: i32,
    pub job_id: String,
    pub keyword: String,
    pub platform: String,
    pub channel_id: String,
    pub title: String,
    pub country: Option<String>,
    pub subscriber_count: Option<i64>,
    pub email: Option<String>,
    pub description: Option<String>,
    pub custom_url: Option<String>,
    pub email_status: String,
    pub enrich_attempts: i32,
    pub enrich_error: Option<String>,
    pub enriched_at: Option<String>,
    pub verified_email: Option<String>,
}
