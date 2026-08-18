//! 账号管理 Tauri commands — 多账号 CRUD + 状态切换。
//!
//! 壳层组合：`InMemoryAccountStore` → `app::account::AccountService`（校验 + 编排）。

use crate::platforms::xianyu::persist::InMemoryAccountStore;
use app::account::{AccountService, AccountStatus, AccountUpdate, XianyuAccount};
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 账号状态变更入参。
#[derive(Debug, Deserialize)]
pub struct AccountStatusRequest {
    pub owner_id: i64,
    pub account_id: String,
    pub status: String,
}

/// 账号删除入参。
#[derive(Debug, Deserialize)]
pub struct AccountDeleteRequest {
    pub owner_id: i64,
    pub account_id: String,
}

/// 账号服务句柄（setup 时注册到 Tauri 状态）。
pub struct AccountHandle {
    pub store: Arc<InMemoryAccountStore>,
}

/// 查询账号列表。
#[tauri::command]
pub fn account_list(
    state: State<'_, AccountHandle>,
    owner_id: i64,
) -> common::OpenDeskResult<Vec<XianyuAccount>> {
    let service = AccountService::new(state.store.as_ref());
    service.list(owner_id).map_err(common::OpenDeskError::wrap)
}

/// 新建账号（含归属/唯一性校验）。
#[tauri::command]
pub fn account_create(
    state: State<'_, AccountHandle>,
    owner_id: i64,
    account: XianyuAccount,
) -> common::OpenDeskResult<XianyuAccount> {
    let service = AccountService::new(state.store.as_ref());
    service
        .create(owner_id, &account)
        .map_err(common::OpenDeskError::wrap)
}

/// 更新账号（部分字段补丁）。
#[tauri::command]
pub fn account_update(
    state: State<'_, AccountHandle>,
    owner_id: i64,
    account_id: String,
    patch: AccountUpdate,
) -> common::OpenDeskResult<XianyuAccount> {
    let service = AccountService::new(state.store.as_ref());
    service
        .update(owner_id, &account_id, &patch)
        .map_err(common::OpenDeskError::wrap)
}

/// 切换账号启用状态。
#[tauri::command]
pub fn account_set_status(
    state: State<'_, AccountHandle>,
    request: AccountStatusRequest,
) -> common::OpenDeskResult<()> {
    let service = AccountService::new(state.store.as_ref());
    let status = AccountStatus::from_str(&request.status);
    service
        .set_status(request.owner_id, &request.account_id, status)
        .map_err(common::OpenDeskError::wrap)
}

/// 删除账号（归属校验）。
#[tauri::command]
pub fn account_delete(
    state: State<'_, AccountHandle>,
    request: AccountDeleteRequest,
) -> common::OpenDeskResult<()> {
    let service = AccountService::new(state.store.as_ref());
    service
        .delete(request.owner_id, &request.account_id)
        .map_err(common::OpenDeskError::wrap)
}
