//! 本地 SQLite 数据文件路径。
//!
//! 作者：coisini
//! 创建时间：2026-07-21

use std::path::PathBuf;

/// `crawler.db` 绝对路径（频道 / 关键词 / crawler settings）。
///
/// 作者：coisini
/// 创建时间：2026-07-21
///
/// # 返回值
/// OpenDesk 数据目录下的 `crawler.db`。
pub fn crawler_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path.push("crawler.db");
    path
}

/// `opendesk.db` 绝对路径（background_job 等协调表）。
///
/// 作者：coisini
/// 创建时间：2026-07-21
///
/// # 返回值
/// OpenDesk 数据目录下的 `opendesk.db`。
pub fn opendesk_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path.push("opendesk.db");
    path
}
