//! 应用启动 / 退出生命周期。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

/// Tauri `setup` 阶段日志 — 具体初始化仍在 `lib.rs` setup 内编排。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
pub fn on_setup() {
    tracing::info!(target: "opendesk.lifecycle", "应用 Rust 壳 setup 完成");
}

/// 应用退出 — 侧车停止由 `lib.rs` `RunEvent::Exit` 调用方负责，此处仅打日志。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
pub fn on_exit() {
    tracing::info!(target: "opendesk.lifecycle", "应用正在退出");
}
