//! 运行日志 IPC 命令 — 供前端日志面板读取/清空进程内日志缓冲。

use crate::logging::{clear_logs, recent_logs, LogEntry};

/// 读取最近日志（时间正序，旧 → 新）。
#[tauri::command]
pub fn log_recent(limit: Option<usize>) -> Vec<LogEntry> {
    recent_logs(limit.unwrap_or(500))
}

/// 清空日志缓冲。
#[tauri::command]
pub fn log_clear() {
    clear_logs();
}
