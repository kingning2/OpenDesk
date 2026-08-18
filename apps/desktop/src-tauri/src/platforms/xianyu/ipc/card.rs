//! 卡券管理 Tauri commands。

use crate::platforms::xianyu::persist::InMemoryCardStore;
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
) -> common::OpenDeskResult<(Vec<Card>, u32)> {
    let service = CardService::new(state.store.as_ref());
    let query = CardQuery {
        page: request.page,
        page_size: request.page_size,
        keyword: request.keyword,
        card_type: request.card_type,
    };
    service.list(request.owner_id, &query)
}

#[tauri::command]
pub fn card_create(
    state: State<'_, CardHandle>,
    owner_id: i64,
    card: Card,
) -> common::OpenDeskResult<Card> {
    let service = CardService::new(state.store.as_ref());
    service
        .create(owner_id, card)
        .map_err(common::OpenDeskError::wrap)
}

#[tauri::command]
pub fn card_update(
    state: State<'_, CardHandle>,
    owner_id: i64,
    card: Card,
) -> common::OpenDeskResult<()> {
    let service = CardService::new(state.store.as_ref());
    service
        .update(owner_id, &card)
        .map_err(common::OpenDeskError::wrap)
}

#[tauri::command]
pub fn card_set_enabled(
    state: State<'_, CardHandle>,
    request: CardEnabledRequest,
) -> common::OpenDeskResult<()> {
    let service = CardService::new(state.store.as_ref());
    service.set_enabled(request.owner_id, request.card_id, request.enabled)
}

#[tauri::command]
pub fn card_delete(
    state: State<'_, CardHandle>,
    request: CardDeleteRequest,
) -> common::OpenDeskResult<()> {
    let service = CardService::new(state.store.as_ref());
    service.delete(request.owner_id, request.card_id)
}
