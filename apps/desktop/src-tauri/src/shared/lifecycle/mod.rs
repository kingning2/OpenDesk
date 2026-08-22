//! 应用级生命周期（与 `infra::sidecar::lifecycle` 无关）。
//!
//! 打开本目录即可对照前端 `apps/desktop/src/lifecycle/`（仅生命周期钩子）：
//!
//! | kind | Rust 入口 |
//! |------|-----------|
//! | `app.start` | [`app::on_setup`] |
//! | `app.exit` | [`app::on_exit`] |
//! | `route.change` | [`route::on_route_change`] |
//! | `ipc.request` | 各 IPC handler / [`crate::shared::ipc::log::log_write`] |
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

pub mod app;
pub mod route;

pub use app::{on_exit, on_setup};
