//! OpenDesk 业务系统操作指引 Skill（页面地图、设置分区、操作指南）。
//!
//! 从 `agent` 中抽出的业务内容，保持 `agent` 为纯 AI 基建。
//! 页面 id 同时供 `navigate_page` 工具校验（见 `apps/desktop/src-tauri/crates/app/src/app/chat_skills.rs`）。

pub mod system;

pub use system::{page_by_id, system_pages, system_registry, SystemPage, SETTINGS_SECTIONS};
