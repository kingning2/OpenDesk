//! 系统导航帮助问答 Tauri command。
//!
//! 帮助页一问一答：注入系统操作指南 Skill + 动作工具（`navigate_page` / `open_settings`），
//! 不落库、无长期记忆、无历史（`store`/`memory`/`embedder` 均传 `None`），
//! 流式事件复用 `chat:message/token` / `chat:message/tool`，`session_id` 固定为 `"help"`。
//!
//! 作者：coisini

use std::sync::Arc;

use chat::{ChatToolCaller, SendChat};
use common::contracts::{ChatIpcSendRequest, HelpIpcAskRequest, HelpIpcAskResponse};

use super::llm::stored_llm_client;
use super::tools_enabled;
use crate::app::chat_emit::TauriChatEmitter;
use crate::app::chat_skills::SkillActionCaller;
use crate::app::state::AppState;

/// 帮助页固定会话 id：前端按此过滤事件，避免与 Chat 页的会话混淆。
const HELP_SESSION_ID: &str = "help";

/// 问一次系统导航问题；回复 token 通过 `chat:message/token` 事件流式推送。
///
/// 每次问答相互独立：不带会话历史、不落库、不使用跨会话长期记忆。
///
/// # 参数
/// - `app` — Tauri app handle（用于事件推送）
/// - `state` — 应用共享状态
/// - `request` — 问答请求（text / message_id）
///
/// # 返回值
/// 含 `message_id` 的确认响应；LLM 未配置或网络失败时返回错误。
#[tauri::command]
pub async fn help_ask(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    request: HelpIpcAskRequest,
) -> Result<HelpIpcAskResponse, String> {
    let client = stored_llm_client(&state).await?;
    let emitter = TauriChatEmitter::new(app);

    let tools = if tools_enabled(&state).await? {
        // 只带动作工具（页面跳转 / 打开设置分区），不暴露数据查询工具。
        Some(Arc::new(SkillActionCaller::new()) as Arc<dyn ChatToolCaller>)
    } else {
        None
    };
    let response = SendChat::execute(
        &client,
        &emitter,
        ChatIpcSendRequest {
            trace_id: None,
            message_id: request.message_id,
            session_id: HELP_SESSION_ID.to_string(),
            messages_json: None,
            text: request.text,
        },
        tools,
        Some(state.skill_registry.clone()),
        None,
        None,
        None,
    )
    .await?;

    Ok(HelpIpcAskResponse {
        ok: true,
        message_id: response.message_id,
        error_message: None,
    })
}
