//! 聊天 Tauri command。
//!
//! 作者：coisini

use std::sync::Arc;

use agent::llm::Strategy;
use chat::{ChatToolCaller, SendChat};
use common::contracts::{
    ChatDtoMessage, ChatDtoSession, ChatIpcMessagesLoadRequest, ChatIpcMessagesLoadResponse,
    ChatIpcSendRequest, ChatIpcSendResponse, ChatIpcSessionCreateRequest,
    ChatIpcSessionCreateResponse, ChatIpcSessionDeleteRequest, ChatIpcSessionDeleteResponse,
    ChatIpcSessionListResponse, ChatIpcSessionRenameRequest, ChatIpcSessionRenameResponse,
};
use ports::chat::{ChatMessageRecord, ChatSessionRecord};

use super::llm::stored_llm_client;
use super::tools_enabled;
use crate::app::chat_emit::TauriChatEmitter;
use crate::app::chat_tools::ChatToolsBridge;
use crate::app::state::AppState;

/// 读取用户是否允许内置 LLM 使用跨会话长期记忆；未配置时按默认开启处理。
async fn memory_enabled(state: &AppState) -> Result<bool, String> {
    let store = state.llm_settings_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        Ok::<_, String>(
            store
                .get()
                .map_err(|error| error.to_string())?
                .map(|record| record.memory_enabled)
                .unwrap_or(true),
        )
    })
    .await
    .map_err(|error| error.to_string())?
}

/// 建立聊天工具：进程内只读数据库 MCP 桥（`agent::mcp` 查询工具）。
/// 系统导航动作工具（`navigate_page` / `open_settings`）已移到 Help 页，不在此提供。
async fn build_chat_tools() -> Result<Arc<dyn ChatToolCaller>, String> {
    let data_dir = agent::mcp::paths::data_dir(None);
    Ok(Arc::new(ChatToolsBridge::new(data_dir).await?))
}

/// 发送一条聊天消息；回复 token 通过 `chat:message/token` 事件流式推送。
///
/// 历史与落库：多会话持久化模式下后端从 `chat_store` 重建历史并在流式过程落库。
///
/// # 参数
/// - `app` — Tauri app handle（用于事件推送）
/// - `state` — 应用共享状态
/// - `request` — 聊天请求（session_id / text）
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
    let allow_memory = memory_enabled(&state).await?;
    let chat_store = state.chat_store.clone();
    let memory = if allow_memory {
        Some(state.chat_memory_store.clone())
    } else {
        None
    };
    let embedder = if allow_memory {
        Some(state.embedder.clone())
    } else {
        None
    };
    let session_id = request.session_id.clone();
    let response = SendChat::execute(
        &client,
        &emitter,
        request,
        tools,
        None,
        Some(chat_store.as_ref()),
        memory,
        embedder,
    )
    .await?;

    // 后台异步生成会话摘要并写入长期记忆，不阻塞本次流式返回。
    if allow_memory {
        let digest_client = client.clone();
        let digest_store = state.chat_store.clone();
        let digest_memory = state.chat_memory_store.clone();
        let digest_embedder = state.embedder.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(error) = chat::maybe_digest(
                &digest_client,
                digest_store.as_ref(),
                digest_memory.as_ref(),
                digest_embedder,
                session_id,
            )
            .await
            {
                tracing::warn!(%error, "chat memory digest failed");
            }
        });
    }

    Ok(response)
}

fn session_to_dto(record: ChatSessionRecord) -> ChatDtoSession {
    ChatDtoSession {
        id: record.id,
        title: record.title,
        created_at: record.created_at,
        updated_at: record.updated_at,
        last_message_at: record.last_message_at,
        message_count: record.message_count,
    }
}

fn message_to_dto(record: ChatMessageRecord) -> ChatDtoMessage {
    ChatDtoMessage {
        id: record.id,
        session_id: record.session_id,
        role: record.role,
        content: record.content,
        thinking: record.thinking,
        tools_json: record.tools_json,
        seq: record.seq,
        created_at: record.created_at,
    }
}

/// 列出全部会话（最近更新的在前）。
///
/// 作者：coisini
#[tauri::command]
pub async fn chat_session_list(
    state: tauri::State<'_, AppState>,
) -> Result<ChatIpcSessionListResponse, String> {
    let store = state.chat_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let sessions = store
            .list_sessions()
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(session_to_dto)
            .collect::<Vec<_>>();
        let sessions_json = serde_json::to_string(&sessions).map_err(|error| error.to_string())?;
        Ok(ChatIpcSessionListResponse { sessions_json })
    })
    .await
    .map_err(|error| error.to_string())?
}

/// 新建一个会话；`title` 可空（首条消息后自动命名）。
///
/// 作者：coisini
#[tauri::command]
pub async fn chat_session_create(
    state: tauri::State<'_, AppState>,
    request: ChatIpcSessionCreateRequest,
) -> Result<ChatIpcSessionCreateResponse, String> {
    let store = state.chat_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let id = uuid::Uuid::new_v4().to_string();
        let session = store
            .create_session(&id, request.title.as_deref().unwrap_or(""))
            .map_err(|error| error.to_string())?;
        let session_json =
            serde_json::to_string(&session_to_dto(session)).map_err(|error| error.to_string())?;
        Ok(ChatIpcSessionCreateResponse { session_json })
    })
    .await
    .map_err(|error| error.to_string())?
}

/// 重命名会话。
///
/// 作者：coisini
#[tauri::command]
pub async fn chat_session_rename(
    state: tauri::State<'_, AppState>,
    request: ChatIpcSessionRenameRequest,
) -> Result<ChatIpcSessionRenameResponse, String> {
    let store = state.chat_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let session = store
            .rename_session(&request.id, &request.title)
            .map_err(|error| error.to_string())?;
        let session_json =
            serde_json::to_string(&session_to_dto(session)).map_err(|error| error.to_string())?;
        Ok(ChatIpcSessionRenameResponse { session_json })
    })
    .await
    .map_err(|error| error.to_string())?
}

/// 删除会话并级联清理其消息。
///
/// 作者：coisini
#[tauri::command]
pub async fn chat_session_delete(
    state: tauri::State<'_, AppState>,
    request: ChatIpcSessionDeleteRequest,
) -> Result<ChatIpcSessionDeleteResponse, String> {
    let store = state.chat_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        store
            .delete_session(&request.id)
            .map_err(|error| error.to_string())?;
        Ok(ChatIpcSessionDeleteResponse { ok: true })
    })
    .await
    .map_err(|error| error.to_string())?
}

/// 加载某会话已落库的完成态消息。
///
/// 作者：coisini
#[tauri::command]
pub async fn chat_messages_load(
    state: tauri::State<'_, AppState>,
    request: ChatIpcMessagesLoadRequest,
) -> Result<ChatIpcMessagesLoadResponse, String> {
    let store = state.chat_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let messages = store
            .load_messages(&request.session_id)
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(message_to_dto)
            .collect::<Vec<_>>();
        let messages_json = serde_json::to_string(&messages).map_err(|error| error.to_string())?;
        Ok(ChatIpcMessagesLoadResponse { messages_json })
    })
    .await
    .map_err(|error| error.to_string())?
}
