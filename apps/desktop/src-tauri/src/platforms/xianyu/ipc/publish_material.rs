//! 商品发布素材 Tauri commands — 素材 CRUD + 批量删除。

use crate::platforms::xianyu::persist::InMemoryPublishMaterialStore;
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
) -> common::OpenDeskResult<(Vec<PublishMaterial>, u32)> {
    let service = PublishMaterialService::new(state.store.as_ref());
    let query = PublishMaterialQuery {
        page: request.page,
        page_size: request.page_size,
        keyword: request.keyword,
        category: request.category,
        condition: request.condition,
        platform_category_id: request.platform_category_id,
    };
    service.list(request.owner_id, &query)
}

#[tauri::command]
pub fn publish_material_create(
    state: State<'_, PublishMaterialHandle>,
    owner_id: i64,
    material: PublishMaterial,
) -> common::OpenDeskResult<PublishMaterial> {
    let service = PublishMaterialService::new(state.store.as_ref());
    service
        .create(owner_id, material)
        .map_err(common::OpenDeskError::wrap)
}

#[tauri::command]
pub fn publish_material_update(
    state: State<'_, PublishMaterialHandle>,
    owner_id: i64,
    material: PublishMaterial,
) -> common::OpenDeskResult<()> {
    let service = PublishMaterialService::new(state.store.as_ref());
    service
        .update(owner_id, &material)
        .map_err(common::OpenDeskError::wrap)
}

#[tauri::command]
pub fn publish_material_delete(
    state: State<'_, PublishMaterialHandle>,
    request: MaterialDeleteRequest,
) -> common::OpenDeskResult<()> {
    let service = PublishMaterialService::new(state.store.as_ref());
    service.delete(request.owner_id, request.material_id)
}

#[tauri::command]
pub fn publish_material_batch_delete(
    state: State<'_, PublishMaterialHandle>,
    request: MaterialBatchDeleteRequest,
) -> common::OpenDeskResult<usize> {
    let service = PublishMaterialService::new(state.store.as_ref());
    service.batch_delete(request.owner_id, &request.material_ids)
}
