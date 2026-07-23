//! Tauri IPC command 按业务域分组。
//!
//! - [`agent`] — Agent / sidecar ping
//! - [`license`] — 授权状态与激活
//! - [`crawler`] — 爬虫 job / 关键词 / settings
//! - [`llm`] — LLM Provider 设置
//! - [`mail`] — 邮件模板 / 账号 / 发信 / 入站记录
//!
//! 作者：coisini
//! 创建时间：2026-07-21

pub mod agent;
pub mod crawler;
pub mod customer;
pub mod license;
pub mod llm;
pub mod mail;
pub mod workflow;

pub use agent::agent_ping;
pub use crawler::{
    crawler_channel_list, crawler_channel_update, crawler_job_cancel, crawler_job_logs,
    crawler_job_results, crawler_job_start, crawler_job_status, crawler_keywords_batches,
    crawler_keywords_import, crawler_youtube_api_key_get, crawler_youtube_api_key_set,
};
pub use customer::{customer_create, customer_get, customer_list, customer_update};
pub use license::{license_activate, license_machine_code, license_status};
pub use llm::{llm_settings_get, llm_settings_save, llm_test_connection};
pub use mail::{
    mail_account_list, mail_account_save, mail_inbox_unmatched_list, mail_link_inbound_customer,
    mail_message_list, mail_record_inbound, mail_send, mail_sync_now, mail_sync_status,
    mail_template_apply, mail_template_list, mail_template_save,
};
pub use workflow::{workflow_snippet_delete, workflow_snippet_list, workflow_snippet_save};
