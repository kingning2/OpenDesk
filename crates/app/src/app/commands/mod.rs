//! Tauri IPC command 按业务域分组。
//!
//! - [`agent`] — Agent / LLM 连通性探测
//! - [`help`] — 系统导航帮助问答
//! - [`license`] — 授权状态与激活
//! - [`crawler`] — 爬虫 job / 关键词 / settings
//! - [`llm`] — LLM Provider 设置
//! - [`mail`] — 邮件模板 / 账号 / 发信 / 入站记录

pub mod agent;
pub mod chat;
pub mod crawler;
pub mod customer;
pub mod dashboard;
pub mod help;
pub mod knowledge;
pub mod license;
pub mod llm;
pub mod mail;
pub mod mail_integration;
pub mod workflow;
pub mod workflow_runtime;

/// 读取用户是否允许内置 LLM 调用工具；未配置时按默认开启处理。
///
/// 供 Chat（数据查询）与 Help（导航动作）两个 command 共用。
pub(crate) async fn tools_enabled(state: &crate::app::state::AppState) -> Result<bool, String> {
    let store = state.llm_settings_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        Ok::<_, String>(
            store
                .get()
                .map_err(|error| error.to_string())?
                .map(|record| record.tools_enabled)
                .unwrap_or(true),
        )
    })
    .await
    .map_err(|error| error.to_string())?
}
