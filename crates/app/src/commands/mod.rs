//! Tauri IPC command 按业务域分组。
//!
//! - [`agent`] — Agent / sidecar ping
//! - [`license`] — 授权状态与激活
//! - [`crawler`] — 爬虫 job / 关键词 / settings
//!
//! 作者：coisini
//! 创建时间：2026-07-21

pub mod agent;
pub mod crawler;
pub mod license;

pub use agent::agent_ping;
pub use crawler::{
    crawler_job_cancel, crawler_job_logs, crawler_job_results, crawler_job_start,
    crawler_job_status, crawler_keywords_batches, crawler_keywords_import,
    crawler_youtube_api_key_get, crawler_youtube_api_key_set,
};
pub use license::{license_activate, license_machine_code, license_status};
