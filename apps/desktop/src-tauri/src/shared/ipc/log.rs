//! 运行日志 IPC 命令 — 供前端日志面板读取/清空/写入进程内日志缓冲。

use crate::logging::{clear_logs, recent_logs, LogEntry};
use crate::shared::ipc::IpcResponse;
use crate::shared::lifecycle::route::on_route_change;

/// 读取最近日志（时间正序，旧 → 新）。
#[tauri::command]
pub fn log_recent(limit: Option<usize>) -> IpcResponse<Vec<LogEntry>> {
    IpcResponse::ok(recent_logs(limit.unwrap_or(500)))
}

/// 清空日志缓冲。
#[tauri::command]
pub fn log_clear() -> IpcResponse<()> {
    clear_logs();
    IpcResponse::ok(())
}

/// 写入一条日志到 Rust tracing 缓冲（供生命周期 / 前端主动上报）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
///
/// # 参数
///
/// * `message` — 日志正文
/// * `level` — 可选级别：TRACE / DEBUG / INFO / WARN / ERROR，默认 INFO
#[tauri::command]
pub fn log_write(message: String, level: Option<String>) -> IpcResponse<()> {
    let level = level.as_deref().unwrap_or("INFO");
    if message.starts_with("访问页面") {
        on_route_change(&message);
        return IpcResponse::ok(());
    }
    match level {
        "ERROR" => tracing::error!(target: "dingda.lifecycle", "{message}"),
        "WARN" => tracing::warn!(target: "dingda.lifecycle", "{message}"),
        "DEBUG" | "TRACE" => tracing::debug!(target: "dingda.lifecycle", "{message}"),
        _ => tracing::info!(target: "dingda.lifecycle", "{message}"),
    }
    IpcResponse::ok(())
}
