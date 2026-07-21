//! License 相关 Tauri commands。
//!
//! 作者：coisini
//! 创建时间：2026-07-21

use common::license::{LicenseActivateRequest, LicenseStatus};

use crate::state::AppState;

/// 查询授权状态 IPC。
///
/// 作者：coisini
/// 创建时间：2026-07-16
///
/// # 参数
/// - `state` — 应用共享状态
///
/// # 返回值
/// 当前 [`LicenseStatus`]。
#[tauri::command]
pub async fn license_status(state: tauri::State<'_, AppState>) -> Result<LicenseStatus, String> {
    state
        .license
        .status()
        .await
        .map_err(|error| error.to_string())
}

/// 读取本机设备码 IPC。
///
/// 作者：coisini
/// 创建时间：2026-07-16
///
/// # 参数
/// - `state` — 应用共享状态
///
/// # 返回值
/// 设备码字符串。
#[tauri::command]
pub async fn license_machine_code(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state
        .license
        .machine_code()
        .await
        .map_err(|error| error.to_string())
}

/// 激活授权 IPC。
///
/// 作者：coisini
/// 创建时间：2026-07-16
///
/// # 参数
/// - `state` — 应用共享状态
/// - `request` — 激活请求（含 license key）
///
/// # 返回值
/// 激活后的 [`LicenseStatus`]。
#[tauri::command]
pub async fn license_activate(
    state: tauri::State<'_, AppState>,
    request: LicenseActivateRequest,
) -> Result<LicenseStatus, String> {
    state
        .license
        .activate(request)
        .await
        .map_err(|error| error.to_string())
}
