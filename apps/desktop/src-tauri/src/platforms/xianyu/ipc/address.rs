//! 发布地址 Tauri commands — 地址 CRUD + 批量删除。

use crate::platforms::xianyu::persist::InMemoryAddressStore;
use app::publish::{AddressQuery, AddressService, PublishAddress};
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 地址服务句柄（setup 时注册到 Tauri 状态）。
pub struct AddressHandle {
    pub store: Arc<InMemoryAddressStore>,
}

#[derive(Debug, Deserialize)]
pub struct AddressListRequest {
    pub owner_id: i64,
    pub page: u32,
    pub page_size: u32,
    #[serde(default)]
    pub keyword: String,
    /// global / personal / 空。
    #[serde(default)]
    pub address_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AddressDeleteRequest {
    pub owner_id: i64,
    pub address_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct AddressBatchDeleteRequest {
    pub owner_id: i64,
    pub address_ids: Vec<i64>,
}

#[tauri::command]
pub fn address_list(
    state: State<'_, AddressHandle>,
    request: AddressListRequest,
) -> common::OpenDeskResult<(Vec<PublishAddress>, u32)> {
    let service = AddressService::new(state.store.as_ref());
    let query = AddressQuery {
        page: request.page,
        page_size: request.page_size,
        keyword: request.keyword,
        address_type: request.address_type,
    };
    service.list(request.owner_id, &query)
}

#[tauri::command]
pub fn address_create(
    state: State<'_, AddressHandle>,
    owner_id: i64,
    address: PublishAddress,
) -> common::OpenDeskResult<PublishAddress> {
    let service = AddressService::new(state.store.as_ref());
    service
        .create(owner_id, address)
        .map_err(common::OpenDeskError::wrap)
}

#[tauri::command]
pub fn address_update(
    state: State<'_, AddressHandle>,
    owner_id: i64,
    address: PublishAddress,
) -> common::OpenDeskResult<()> {
    let service = AddressService::new(state.store.as_ref());
    service
        .update(owner_id, &address)
        .map_err(common::OpenDeskError::wrap)
}

#[tauri::command]
pub fn address_delete(
    state: State<'_, AddressHandle>,
    request: AddressDeleteRequest,
) -> common::OpenDeskResult<()> {
    let service = AddressService::new(state.store.as_ref());
    service.delete(request.owner_id, request.address_id)
}

#[tauri::command]
pub fn address_batch_delete(
    state: State<'_, AddressHandle>,
    request: AddressBatchDeleteRequest,
) -> common::OpenDeskResult<usize> {
    let service = AddressService::new(state.store.as_ref());
    service.batch_delete(request.owner_id, &request.address_ids)
}
