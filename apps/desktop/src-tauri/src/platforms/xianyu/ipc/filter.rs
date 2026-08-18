//! 消息过滤规则 Tauri commands。

use crate::platforms::xianyu::persist::InMemoryFilterStore;
use crate::shared::ipc::IpcResponse;
use app::auto_reply::filter::FilterRule;
use app::auto_reply::FilterService;
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 过滤规则服务句柄（setup 时注册到 Tauri 状态）。
pub struct FilterHandle {
    pub store: Arc<InMemoryFilterStore>,
}

#[derive(Debug, Deserialize)]
pub struct FilterListRequest {
    pub owner_id: i64,
    pub account_id: String,
}

#[derive(Debug, Deserialize)]
pub struct FilterCreateRequest {
    pub owner_id: i64,
    pub account_id: String,
    pub rule: FilterRule,
}

#[derive(Debug, Deserialize)]
pub struct FilterUpdateRequest {
    pub owner_id: i64,
    pub rule: FilterRule,
}

#[derive(Debug, Deserialize)]
pub struct FilterEnabledRequest {
    pub owner_id: i64,
    pub rule_id: i64,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct FilterDeleteRequest {
    pub owner_id: i64,
    pub rule_id: i64,
}

#[tauri::command]
pub fn filter_list(
    state: State<'_, FilterHandle>,
    request: FilterListRequest,
) -> common::DingDaResult<IpcResponse<Vec<FilterRule>>> {
    let service = FilterService::new(state.store.as_ref());
    let result = service.list(request.owner_id, &request.account_id)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn filter_create(
    state: State<'_, FilterHandle>,
    request: FilterCreateRequest,
) -> common::DingDaResult<IpcResponse<FilterRule>> {
    let service = FilterService::new(state.store.as_ref());
    let result = service.create(request.owner_id, &request.account_id, request.rule)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn filter_update(
    state: State<'_, FilterHandle>,
    request: FilterUpdateRequest,
) -> common::DingDaResult<IpcResponse<()>> {
    let service = FilterService::new(state.store.as_ref());
    service
        .update(request.owner_id, &request.rule)
        .map_err(common::DingDaError::wrap)?;
    Ok(IpcResponse::ok(()))
}

#[tauri::command]
pub fn filter_set_enabled(
    state: State<'_, FilterHandle>,
    request: FilterEnabledRequest,
) -> common::DingDaResult<IpcResponse<()>> {
    let service = FilterService::new(state.store.as_ref());
    service.set_enabled(request.owner_id, request.rule_id, request.enabled)?;
    Ok(IpcResponse::ok(()))
}

#[tauri::command]
pub fn filter_delete(
    state: State<'_, FilterHandle>,
    request: FilterDeleteRequest,
) -> common::DingDaResult<IpcResponse<()>> {
    let service = FilterService::new(state.store.as_ref());
    service.delete(request.owner_id, request.rule_id)?;
    Ok(IpcResponse::ok(()))
}
