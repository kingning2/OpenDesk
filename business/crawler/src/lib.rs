//! 桌面端进程内 YouTube 采集与邮箱补全能力。
//!
//! - [`crawler::youtube`](youtube) — YouTube Data API 采集（始终编译，主进程使用）。
//! - [`crawler::enrich`](enrich) — 邮箱补全 RPA（仅 `enrich` feature 编译，worker 使用）。

mod email;

pub mod youtube;

#[cfg(feature = "enrich")]
pub mod enrich;
