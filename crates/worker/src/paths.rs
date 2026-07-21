//! Resolve desktop data directory paths for Worker databases.
//!
//! 作者：coisini
//! 创建时间：2026-07-20

use std::path::PathBuf;

/// Return `{data_local}/OpenDesk/opendesk.db`.
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn opendesk_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path.push("opendesk.db");
    path
}

/// Return `{data_local}/OpenDesk/crawler.db`.
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn crawler_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path.push("crawler.db");
    path
}
