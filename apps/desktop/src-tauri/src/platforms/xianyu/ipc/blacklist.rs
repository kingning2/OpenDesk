//! 黑名单管理 Tauri commands。

use crate::platforms::xianyu::persist::InMemoryBlacklistStore;
use crate::shared::ipc::IpcResponse;
use app::blacklist::{
    BlacklistQuery, BlacklistService, PersonalBlacklistItem, PlatformBlacklistItem,
};
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

pub struct BlacklistHandle {
    pub store: Arc<InMemoryBlacklistStore>,
}

#[derive(Debug, Deserialize)]
pub struct PersonalBlacklistListRequest {
    pub owner_id: i64,
    pub page: u32,
    pub page_size: u32,
    #[serde(default)]
    pub buyer_id: String,
    #[serde(default)]
    pub buyer_nick: String,
}

#[derive(Debug, Deserialize)]
pub struct PersonalBlacklistCreateRequest {
    pub owner_id: i64,
    pub buyer_ids: String,
    pub account_id: Option<String>,
    pub item_id: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BlacklistEnabledRequest {
    pub owner_id: i64,
    pub id: i64,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct BlacklistDeleteRequest {
    pub owner_id: i64,
    pub id: i64,
}

#[tauri::command]
pub fn blacklist_personal_list(
    state: State<'_, BlacklistHandle>,
    request: PersonalBlacklistListRequest,
) -> common::DingDaResult<IpcResponse<(Vec<PersonalBlacklistItem>, u32)>> {
    let service = BlacklistService::new(state.store.as_ref());
    let query = BlacklistQuery {
        page: request.page,
        page_size: request.page_size,
        buyer_id: request.buyer_id,
        buyer_nick: request.buyer_nick,
    };
    let result = service
        .list_personal(request.owner_id, &query)
        .map_err(common::DingDaError::wrap)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn blacklist_platform_list(
    state: State<'_, BlacklistHandle>,
    owner_id: i64,
) -> common::DingDaResult<IpcResponse<(Vec<PlatformBlacklistItem>, u32)>> {
    let service = BlacklistService::new(state.store.as_ref());
    let result = service
        .list_platform(owner_id, &BlacklistQuery::default())
        .map_err(common::DingDaError::wrap)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn blacklist_personal_create(
    state: State<'_, BlacklistHandle>,
    request: PersonalBlacklistCreateRequest,
) -> common::DingDaResult<IpcResponse<Vec<PersonalBlacklistItem>>> {
    let service = BlacklistService::new(state.store.as_ref());
    let ids: Vec<&str> = request
        .buyer_ids
        .split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .collect();
    if ids.is_empty() {
        return Err("买家 ID 不能为空".into());
    }
    let mut created = Vec::with_capacity(ids.len());
    for buyer_id in ids {
        let item = service
            .create(
                request.owner_id,
                buyer_id,
                request.account_id.as_deref(),
                request.item_id.as_deref(),
                request.reason.as_deref(),
            )
            .map_err(common::DingDaError::wrap)?;
        created.push(item);
    }
    Ok(IpcResponse::ok(created))
}

#[tauri::command]
pub fn blacklist_set_enabled(
    state: State<'_, BlacklistHandle>,
    request: BlacklistEnabledRequest,
) -> common::DingDaResult<IpcResponse<()>> {
    let service = BlacklistService::new(state.store.as_ref());
    service
        .set_enabled(request.owner_id, request.id, request.enabled)
        .map_err(common::DingDaError::wrap)?;
    Ok(IpcResponse::ok(()))
}

#[tauri::command]
pub fn blacklist_delete(
    state: State<'_, BlacklistHandle>,
    request: BlacklistDeleteRequest,
) -> common::DingDaResult<IpcResponse<()>> {
    let service = BlacklistService::new(state.store.as_ref());
    service
        .delete(request.owner_id, request.id)
        .map_err(common::DingDaError::wrap)?;
    Ok(IpcResponse::ok(()))
}
