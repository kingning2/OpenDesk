//! Tauri IPC command 按业务域分组。
//!
//! - [`agent`] — Agent / LLM 连通性探测
//! - [`license`] — 授权状态与激活
//! - [`crawler`] — 爬虫 job / 关键词 / settings
//! - [`llm`] — LLM Provider 设置
//! - [`mail`] — 邮件模板 / 账号 / 发信 / 入站记录

pub mod agent;
pub mod crawler;
pub mod customer;
pub mod license;
pub mod llm;
pub mod mail;
pub mod mail_integration;
pub mod workflow;
