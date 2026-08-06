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

/// `chat.db` 绝对路径（会话 / 消息 / 记忆）。
///
/// chat.db 用 rusqlite + sqlite-vec 管理（向量扩展无法走 Diesel），与 opendesk.db 分开。
///
/// # 返回值
/// OpenDesk 数据目录下的 `chat.db`。
pub fn chat_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path.push("chat.db");
    path
}

/// fastembed 本地模型缓存目录（首次运行联网下载后完全离线）。
///
/// # 返回值
/// OpenDesk 数据目录下的 `.fastembed_cache`。
pub fn embedding_cache_dir() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path.push(".fastembed_cache");
    path
}

/// `knowledge.db` 绝对路径（知识库文档 / 分块 / 向量）。
///
/// 与 chat.db 一致使用 rusqlite + sqlite-vec 管理（向量扩展无法走 Diesel）。
///
/// # 返回值
/// OpenDesk 数据目录下的 `knowledge.db`。
pub fn knowledge_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path.push("knowledge.db");
    path
}
