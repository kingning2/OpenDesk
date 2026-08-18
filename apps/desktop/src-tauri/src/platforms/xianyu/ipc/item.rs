//! 商品管理 Tauri commands。

use crate::platforms::xianyu::persist::InMemoryItemStore;
use crate::shared::ipc::IpcResponse;
use app::item::{Item, ItemQuery, ItemService};
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

pub struct ItemHandle {
    pub store: Arc<InMemoryItemStore>,
}

#[derive(Debug, Deserialize)]
pub struct ItemListRequest {
    pub owner_id: i64,
    pub page: u32,
    pub page_size: u32,
    #[serde(default)]
    pub keyword: String,
    #[serde(default)]
    pub account_id: String,
    pub is_polished: Option<bool>,
    pub is_multi_spec: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ItemUpdateRequest {
    pub owner_id: i64,
    pub item_id: String,
    pub ai_prompt: Option<String>,
}

#[tauri::command]
pub fn item_list(
    state: State<'_, ItemHandle>,
    request: ItemListRequest,
) -> common::OpenDeskResult<IpcResponse<(Vec<Item>, u32)>> {
    let service = ItemService::new(state.store.as_ref());
    let query = ItemQuery {
        page: request.page,
        page_size: request.page_size,
        keyword: request.keyword,
        account_id: request.account_id,
        is_polished: request.is_polished,
        is_multi_spec: request.is_multi_spec,
    };
    let result = service
        .list(request.owner_id, &query)
        .map_err(common::OpenDeskError::wrap)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn item_get(
    state: State<'_, ItemHandle>,
    owner_id: i64,
    item_id: String,
) -> common::OpenDeskResult<IpcResponse<Option<Item>>> {
    let service = ItemService::new(state.store.as_ref());
    let result = service
        .get(owner_id, &item_id)
        .map_err(common::OpenDeskError::wrap)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn item_update(
    state: State<'_, ItemHandle>,
    request: ItemUpdateRequest,
) -> common::OpenDeskResult<IpcResponse<()>> {
    let service = ItemService::new(state.store.as_ref());
    service
        .update(request.owner_id, &request.item_id, |item| {
            if let Some(ai_prompt) = &request.ai_prompt {
                item.ai_prompt = ai_prompt.clone();
            }
        })
        .map_err(common::OpenDeskError::wrap)?;
    Ok(IpcResponse::ok(()))
}
