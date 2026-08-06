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

/// Return `{data_local}/OpenDesk/opendesk-worker.lock` — 单实例文件锁路径。
///
/// 作者：coisini
/// 创建时间：2026-08-06
pub fn worker_lock_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path.push("opendesk-worker.lock");
    path
}

/// Return `{data_local}/OpenDesk/knowledge.db` — 知识库文档 / 分块 / 向量。
pub fn knowledge_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path.push("knowledge.db");
    path
}

/// Return `{data_local}/OpenDesk/.fastembed_cache` — 本地嵌入模型缓存。
pub fn embedding_cache_dir() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path.push(".fastembed_cache");
    path
}
