//! 应用版本 IPC。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-20

use common::DingDaResult;
use tauri::AppHandle;

use crate::shared::ipc::IpcResponse;

/// 读取当前应用版本（与 `tauri.conf.json` / Cargo 版本一致）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-20
///
/// # 参数
///
/// * `app` — Tauri 应用句柄
///
/// # 返回值
///
/// 语义化版本字符串，如 `0.1.0`。
#[tauri::command]
pub fn app_version(app: AppHandle) -> DingDaResult<IpcResponse<String>> {
    Ok(IpcResponse::ok(app.package_info().version.to_string()))
}
