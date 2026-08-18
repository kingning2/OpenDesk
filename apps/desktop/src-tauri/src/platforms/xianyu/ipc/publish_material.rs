//! 商品发布素材 Tauri commands — 素材 CRUD + 批量删除。

use crate::platforms::xianyu::persist::InMemoryPublishMaterialStore;
use crate::shared::ipc::IpcResponse;
use app::publish::{PublishMaterial, PublishMaterialQuery, PublishMaterialService};
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 素材服务句柄（setup 时注册到 Tauri 状态）。
pub struct PublishMaterialHandle {
    pub store: Arc<InMemoryPublishMaterialStore>,
}

#[derive(Debug, Deserialize)]
pub struct MaterialListRequest {
    pub owner_id: i64,
    pub page: u32,
    pub page_size: u32,
    #[serde(default)]
    pub keyword: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub condition: String,
    #[serde(default)]
    pub platform_category_id: String,
}

#[derive(Debug, Deserialize)]
pub struct MaterialDeleteRequest {
    pub owner_id: i64,
    pub material_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct MaterialBatchDeleteRequest {
    pub owner_id: i64,
    pub material_ids: Vec<i64>,
}

#[tauri::command]
pub fn publish_material_list(
    state: State<'_, PublishMaterialHandle>,
    request: MaterialListRequest,
) -> common::DingDaResult<IpcResponse<(Vec<PublishMaterial>, u32)>> {
    let service = PublishMaterialService::new(state.store.as_ref());
    let query = PublishMaterialQuery {
        page: request.page,
        page_size: request.page_size,
        keyword: request.keyword,
        category: request.category,
        condition: request.condition,
        platform_category_id: request.platform_category_id,
    };
    let result = service.list(request.owner_id, &query)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn publish_material_create(
    state: State<'_, PublishMaterialHandle>,
    owner_id: i64,
    material: PublishMaterial,
) -> common::DingDaResult<IpcResponse<PublishMaterial>> {
    let service = PublishMaterialService::new(state.store.as_ref());
    let result = service
        .create(owner_id, material)
        .map_err(common::DingDaError::wrap)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn publish_material_update(
    state: State<'_, PublishMaterialHandle>,
    owner_id: i64,
    material: PublishMaterial,
) -> common::DingDaResult<IpcResponse<()>> {
    let service = PublishMaterialService::new(state.store.as_ref());
    service
        .update(owner_id, &material)
        .map_err(common::DingDaError::wrap)?;
    Ok(IpcResponse::ok(()))
}

#[tauri::command]
pub fn publish_material_delete(
    state: State<'_, PublishMaterialHandle>,
    request: MaterialDeleteRequest,
) -> common::DingDaResult<IpcResponse<()>> {
    let service = PublishMaterialService::new(state.store.as_ref());
    service.delete(request.owner_id, request.material_id)?;
    Ok(IpcResponse::ok(()))
}

#[tauri::command]
pub fn publish_material_batch_delete(
    state: State<'_, PublishMaterialHandle>,
    request: MaterialBatchDeleteRequest,
) -> common::DingDaResult<IpcResponse<usize>> {
    let service = PublishMaterialService::new(state.store.as_ref());
    let result = service.batch_delete(request.owner_id, &request.material_ids)?;
    Ok(IpcResponse::ok(result))
}
