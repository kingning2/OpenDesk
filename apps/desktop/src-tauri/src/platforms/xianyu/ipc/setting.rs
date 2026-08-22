//! 用户设置 Tauri commands — 用户级键值存储。

use crate::platforms::xianyu::persist::InMemoryUserSettingStore;
use crate::shared::ipc::IpcResponse;
use common;
use platform::domain::setting::UserSettingService;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 用户设置服务句柄（setup 时注册到 Tauri 状态）。
pub struct UserSettingHandle {
    pub store: Arc<InMemoryUserSettingStore>,
}

/// 单键读取请求。
#[derive(Debug, Deserialize)]
pub struct SettingGetRequest {
    pub owner_id: i64,
    pub key: String,
}

/// 单键写入请求。
#[derive(Debug, Deserialize)]
pub struct SettingSetRequest {
    pub owner_id: i64,
    pub key: String,
    pub value: String,
}

/// 读取单键设置。
#[tauri::command]
pub fn user_setting_get(
    state: State<'_, UserSettingHandle>,
    request: SettingGetRequest,
) -> common::DingDaResult<IpcResponse<Option<String>>> {
    let service = UserSettingService::new(state.store.as_ref());
    let result = service.get(request.owner_id, &request.key)?;
    Ok(IpcResponse::ok(result))
}

/// 写入单键设置。
#[tauri::command]
pub fn user_setting_set(
    state: State<'_, UserSettingHandle>,
    request: SettingSetRequest,
) -> common::DingDaResult<IpcResponse<()>> {
    let service = UserSettingService::new(state.store.as_ref());
    service.set(request.owner_id, &request.key, &request.value)?;
    Ok(IpcResponse::ok(()))
}
