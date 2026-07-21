//! Customer profile Tauri IPC commands.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use common::contracts::{
    CustomerIpcCreateRequest, CustomerIpcCreateResponse, CustomerIpcGetRequest,
    CustomerIpcGetResponse, CustomerIpcListRequest, CustomerIpcListResponse,
    CustomerIpcUpdateRequest, CustomerIpcUpdateResponse,
};
use customer::app::{CreateCustomer, GetCustomer, ListCustomers, UpdateCustomer};

use crate::state::AppState;

/// List customers with optional search and pagination.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[tauri::command]
pub async fn customer_list(
    state: tauri::State<'_, AppState>,
    request: CustomerIpcListRequest,
) -> Result<CustomerIpcListResponse, String> {
    let store = state.customer_store.clone();
    tauri::async_runtime::spawn_blocking(move || ListCustomers::execute(store.as_ref(), request))
        .await
        .map_err(|error| error.to_string())?
}

/// Fetch one customer profile by id.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[tauri::command]
pub async fn customer_get(
    state: tauri::State<'_, AppState>,
    request: CustomerIpcGetRequest,
) -> Result<CustomerIpcGetResponse, String> {
    let store = state.customer_store.clone();
    tauri::async_runtime::spawn_blocking(move || GetCustomer::execute(store.as_ref(), request))
        .await
        .map_err(|error| error.to_string())?
}

/// Create a new customer profile.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[tauri::command]
pub async fn customer_create(
    state: tauri::State<'_, AppState>,
    request: CustomerIpcCreateRequest,
) -> Result<CustomerIpcCreateResponse, String> {
    let store = state.customer_store.clone();
    tauri::async_runtime::spawn_blocking(move || CreateCustomer::execute(store.as_ref(), request))
        .await
        .map_err(|error| error.to_string())?
}

/// Update an existing customer profile.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[tauri::command]
pub async fn customer_update(
    state: tauri::State<'_, AppState>,
    request: CustomerIpcUpdateRequest,
) -> Result<CustomerIpcUpdateResponse, String> {
    let store = state.customer_store.clone();
    tauri::async_runtime::spawn_blocking(move || UpdateCustomer::execute(store.as_ref(), request))
        .await
        .map_err(|error| error.to_string())?
}
