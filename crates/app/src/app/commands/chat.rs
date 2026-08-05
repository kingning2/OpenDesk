//! 聊天 Tauri command。
//!
//! 作者：coisini

use std::sync::Arc;

use agent::llm::Strategy;
use chat::{ChatToolCaller, SendChat};
use common::contracts::{ChatIpcSendRequest, ChatIpcSendResponse};

use super::llm::stored_llm_client;
use crate::app::chat_emit::TauriChatEmitter;
use crate::app::chat_tools::ChatToolsBridge;
use crate::app::state::AppState;

/// 读取用户是否允许内置 LLM 调用数据查询工具；未配置时按默认开启处理。
async fn tools_enabled(state: &AppState) -> Result<bool, String> {
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

/// 建立进程内只读数据库 MCP 桥（复用 `opendesk-mcp` 的 4 个查询工具）。
async fn build_chat_tools() -> Result<Arc<dyn ChatToolCaller>, String> {
    let data_dir = opendesk_mcp::paths::data_dir(None);
    Ok(Arc::new(ChatToolsBridge::new(data_dir).await?))
}

/// 发送一条聊天消息；回复 token 通过 `chat:message/token` 事件流式推送。
///
/// # 参数
/// - `app` — Tauri app handle（用于事件推送）
/// - `state` — 应用共享状态
/// - `request` — 聊天请求（session_id / messages_json / text）
///
/// # 返回值
/// 含 `message_id` 的确认响应；LLM 未配置或网络失败时返回错误。
#[tauri::command]
pub async fn chat_send(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    request: ChatIpcSendRequest,
) -> Result<ChatIpcSendResponse, String> {
    tracing::info!(
        session_id = %request.session_id,
        message_id = %request.message_id.as_deref().unwrap_or("-"),
        text_chars = request.text.chars().count(),
        "chat_send: request received"
    );
    let client = stored_llm_client(&state).await?;
    let emitter = TauriChatEmitter::new(app);

    let allow_tools = tools_enabled(&state).await?;
    let tools = if allow_tools && client.strategy() == Strategy::OpenAiCompatible {
        Some(build_chat_tools().await?)
    } else {
        None
    };
    SendChat::execute(&client, &emitter, request, tools).await
}
