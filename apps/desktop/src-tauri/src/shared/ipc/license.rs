//! License ?? Tauri commands?
//!
//! ???coisini
//! ?????2026-07-21

use common::license::{LicenseActivateRequest, LicenseStatus};
use common::DingDaResult;
use dingda_macros::timed;

use crate::shared::ipc::IpcResponse;
use crate::state::AppState;

/// ?????? IPC?
///
/// ???coisini
/// ?????2026-07-16
///
/// # ??
/// - `state` ? ??????
///
/// # ???
/// ?? [`LicenseStatus`]?
#[tauri::command]
#[timed]
pub async fn license_status(
    state: tauri::State<'_, AppState>,
) -> DingDaResult<IpcResponse<LicenseStatus>> {
    Ok(IpcResponse::ok(
        state
            .license
            .status()
            .await
            .map_err(|error| error.to_string())?,
    ))
}

/// ??????? IPC?
///
/// ???coisini
/// ?????2026-07-16
///
/// # ??
/// - `state` ? ??????
///
/// # ???
/// ???????
#[tauri::command]
#[timed]
pub async fn license_machine_code(
    state: tauri::State<'_, AppState>,
) -> DingDaResult<IpcResponse<String>> {
    Ok(IpcResponse::ok(
        state
            .license
            .machine_code()
            .await
            .map_err(|error| error.to_string())?,
    ))
}

/// ???? IPC?
///
/// ???coisini
/// ?????2026-07-16
///
/// # ??
/// - `state` ? ??????
/// - `request` ? ?????? license key?
///
/// # ???
/// ???? [`LicenseStatus`]?
#[tauri::command]
#[timed]
pub async fn license_activate(
    state: tauri::State<'_, AppState>,
    request: LicenseActivateRequest,
) -> DingDaResult<IpcResponse<LicenseStatus>> {
    Ok(IpcResponse::ok(
        state
            .license
            .activate(request)
            .await
            .map_err(|error| error.to_string())?,
    ))
}
