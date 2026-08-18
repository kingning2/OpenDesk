//! 卡券管理 Tauri commands。

use crate::platforms::xianyu::persist::InMemoryCardStore;
use crate::shared::ipc::IpcResponse;
use app::card::{CardQuery, CardService};
use app::delivery::execution::card::Card;
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 卡券服务句柄（setup 时注册到 Tauri 状态）。
pub struct CardHandle {
    pub store: Arc<InMemoryCardStore>,
}

#[derive(Debug, Deserialize)]
pub struct CardListRequest {
    pub owner_id: i64,
    pub page: u32,
    pub page_size: u32,
    #[serde(default)]
    pub keyword: String,
    #[serde(default)]
    pub card_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CardEnabledRequest {
    pub owner_id: i64,
    pub card_id: i64,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CardDeleteRequest {
    pub owner_id: i64,
    pub card_id: i64,
}

#[tauri::command]
pub fn card_list(
    state: State<'_, CardHandle>,
    request: CardListRequest,
) -> common::OpenDeskResult<IpcResponse<(Vec<Card>, u32)>> {
    let service = CardService::new(state.store.as_ref());
    let query = CardQuery {
        page: request.page,
        page_size: request.page_size,
        keyword: request.keyword,
        card_type: request.card_type,
    };
    let result = service.list(request.owner_id, &query)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn card_create(
    state: State<'_, CardHandle>,
    owner_id: i64,
    card: Card,
) -> common::OpenDeskResult<IpcResponse<Card>> {
    let service = CardService::new(state.store.as_ref());
    let result = service
        .create(owner_id, card)
        .map_err(common::OpenDeskError::wrap)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn card_update(
    state: State<'_, CardHandle>,
    owner_id: i64,
    card: Card,
) -> common::OpenDeskResult<IpcResponse<()>> {
    let service = CardService::new(state.store.as_ref());
    service
        .update(owner_id, &card)
        .map_err(common::OpenDeskError::wrap)?;
    Ok(IpcResponse::ok(()))
}

#[tauri::command]
pub fn card_set_enabled(
    state: State<'_, CardHandle>,
    request: CardEnabledRequest,
) -> common::OpenDeskResult<IpcResponse<()>> {
    let service = CardService::new(state.store.as_ref());
    service.set_enabled(request.owner_id, request.card_id, request.enabled)?;
    Ok(IpcResponse::ok(()))
}

#[tauri::command]
pub fn card_delete(
    state: State<'_, CardHandle>,
    request: CardDeleteRequest,
) -> common::OpenDeskResult<IpcResponse<()>> {
    let service = CardService::new(state.store.as_ref());
    service.delete(request.owner_id, request.card_id)?;
    Ok(IpcResponse::ok(()))
}
