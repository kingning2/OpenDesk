//! Workflow Tauri IPC commands — script snippet CRUD.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use common::contracts::{
    WorkflowIpcSnippetDeleteRequest, WorkflowIpcSnippetListRequest, WorkflowIpcSnippetListResponse,
    WorkflowIpcSnippetSaveRequest,
};
use ports::workflow::{ScriptSnippetStore, ScriptSnippetWriteInput};

use crate::state::AppState;

/// List script snippets with optional category / free-text filters.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
///
/// # 参数
/// - `request` — optional category_l1, category_l2, query filters
///
/// # 返回值
/// Serialised snippet list with total count.
#[tauri::command]
pub async fn workflow_snippet_list(
    state: tauri::State<'_, AppState>,
    request: WorkflowIpcSnippetListRequest,
) -> Result<WorkflowIpcSnippetListResponse, String> {
    let store = state.snippet_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let records = store
            .list(
                request.category_l1.as_deref(),
                request.category_l2.as_deref(),
                request.query.as_deref(),
            )
            .map_err(|e| e.to_string())?;

        let total = records.len() as i64;
        let snippets_json = serde_json::to_string(
            &records
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "source_id": r.source_id,
                        "title": r.title,
                        "stage": r.stage,
                        "trigger_text": r.trigger_text,
                        "description": r.description,
                        "from_stage": r.from_stage,
                        "to_stage": r.to_stage,
                        "tags_json": r.tags_json,
                        "body_text": r.body_text,
                        "category_l1": r.category_l1,
                        "category_l2": r.category_l2,
                        "needs_boss_input": r.needs_boss_input,
                        "boss_input_hint": r.boss_input_hint,
                        "sort_order": r.sort_order,
                        "created_at": r.created_at,
                        "updated_at": r.updated_at,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())?;

        Ok(WorkflowIpcSnippetListResponse {
            snippets_json,
            total,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Create or update a script snippet.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
///
/// # 参数
/// - `request` — snippet fields; omit `id` to create
///
/// # 返回值
/// Updated list of all snippets matching the same category filters.
#[tauri::command]
pub async fn workflow_snippet_save(
    state: tauri::State<'_, AppState>,
    request: WorkflowIpcSnippetSaveRequest,
) -> Result<WorkflowIpcSnippetListResponse, String> {
    let store = state.snippet_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let input = ScriptSnippetWriteInput {
            id: request.id,
            title: request.title,
            stage: request.stage.clone(),
            trigger_text: request.trigger_text,
            description: request.description,
            from_stage: request.from_stage,
            to_stage: request.to_stage,
            tags_json: request.tags_json,
            body_text: request.body_text,
            category_l1: request.category_l1.clone(),
            category_l2: request.category_l2.clone(),
            needs_boss_input: request.needs_boss_input,
            boss_input_hint: request.boss_input_hint,
            sort_order: request.sort_order,
        };
        store.save(input).map_err(|e| e.to_string())?;

        let records = store
            .list(
                request.category_l1.as_deref(),
                request.category_l2.as_deref(),
                None,
            )
            .map_err(|e| e.to_string())?;

        let total = records.len() as i64;
        let snippets_json = serde_json::to_string(
            &records
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "source_id": r.source_id,
                        "title": r.title,
                        "stage": r.stage,
                        "trigger_text": r.trigger_text,
                        "description": r.description,
                        "from_stage": r.from_stage,
                        "to_stage": r.to_stage,
                        "tags_json": r.tags_json,
                        "body_text": r.body_text,
                        "category_l1": r.category_l1,
                        "category_l2": r.category_l2,
                        "needs_boss_input": r.needs_boss_input,
                        "boss_input_hint": r.boss_input_hint,
                        "sort_order": r.sort_order,
                        "created_at": r.created_at,
                        "updated_at": r.updated_at,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())?;

        Ok(WorkflowIpcSnippetListResponse {
            snippets_json,
            total,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Delete a script snippet by id.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
///
/// # 参数
/// - `request.id` — id of snippet to delete
///
/// # 返回值
/// Updated list (no filters).
#[tauri::command]
pub async fn workflow_snippet_delete(
    state: tauri::State<'_, AppState>,
    request: WorkflowIpcSnippetDeleteRequest,
) -> Result<WorkflowIpcSnippetListResponse, String> {
    let store = state.snippet_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        store.delete(&request.id).map_err(|e| e.to_string())?;

        let records = store.list(None, None, None).map_err(|e| e.to_string())?;
        let total = records.len() as i64;
        let snippets_json = serde_json::to_string(
            &records
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "source_id": r.source_id,
                        "title": r.title,
                        "stage": r.stage,
                        "trigger_text": r.trigger_text,
                        "description": r.description,
                        "from_stage": r.from_stage,
                        "to_stage": r.to_stage,
                        "tags_json": r.tags_json,
                        "body_text": r.body_text,
                        "category_l1": r.category_l1,
                        "category_l2": r.category_l2,
                        "needs_boss_input": r.needs_boss_input,
                        "boss_input_hint": r.boss_input_hint,
                        "sort_order": r.sort_order,
                        "created_at": r.created_at,
                        "updated_at": r.updated_at,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())?;

        Ok(WorkflowIpcSnippetListResponse {
            snippets_json,
            total,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
