//! OpenDesk 无 Tauri 的应用状态组装核心。
//!
//! 桌面 Tauri 进程与独立的 HTTP server 进程共用同一套 store / 服务装配，
//! 这里把与 Tauri 无关的部分抽取出来，保证两进程打开同一批 SQLite 文件、
//! 构建同一套 `AppState`。

pub mod paths;
pub mod state;

pub use state::{build_app_state, build_license_gate, AppState};
