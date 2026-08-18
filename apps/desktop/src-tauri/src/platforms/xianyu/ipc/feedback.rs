//! 意见反馈 Tauri commands。

use crate::platforms::xianyu::persist::InMemoryFeedbackStore;
use crate::shared::ipc::IpcResponse;
use app::feedback::{Feedback, FeedbackQuery, FeedbackService};
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 反馈服务句柄（setup 时注册到 Tauri 状态）。
pub struct FeedbackHandle {
    pub store: Arc<InMemoryFeedbackStore>,
}

#[derive(Debug, Deserialize)]
pub struct FeedbackListRequest {
    pub owner_id: i64,
    pub page: u32,
    pub page_size: u32,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub keyword: String,
}

#[derive(Debug, Deserialize)]
pub struct FeedbackDeleteRequest {
    pub owner_id: i64,
    pub feedback_id: i64,
}

#[tauri::command]
pub fn feedback_list(
    state: State<'_, FeedbackHandle>,
    request: FeedbackListRequest,
) -> common::OpenDeskResult<IpcResponse<(Vec<Feedback>, u32)>> {
    let service = FeedbackService::new(state.store.as_ref());
    let query = FeedbackQuery {
        page: request.page,
        page_size: request.page_size,
        kind: request.kind,
        keyword: request.keyword,
    };
    let result = service.list(request.owner_id, &query)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn feedback_create(
    state: State<'_, FeedbackHandle>,
    owner_id: i64,
    feedback: Feedback,
) -> common::OpenDeskResult<IpcResponse<Feedback>> {
    let service = FeedbackService::new(state.store.as_ref());
    let result = service
        .create(owner_id, feedback)
        .map_err(common::OpenDeskError::wrap)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn feedback_delete(
    state: State<'_, FeedbackHandle>,
    request: FeedbackDeleteRequest,
) -> common::OpenDeskResult<IpcResponse<()>> {
    let service = FeedbackService::new(state.store.as_ref());
    service.delete(request.owner_id, request.feedback_id)?;
    Ok(IpcResponse::ok(()))
}
