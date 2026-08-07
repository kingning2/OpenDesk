//! OpenDesk 数据目录与数据库路径解析。
//!
//! 默认 `{data_local}/OpenDesk`；可通过 `OPENDESK_DATA_DIR` 环境变量整体覆盖。

use std::path::PathBuf;

/// OpenDesk 数据目录：`OPENDESK_DATA_DIR` 环境变量 > `{data_local}/OpenDesk`。
pub fn opendesk_data_dir() -> PathBuf {
    std::env::var("OPENDESK_DATA_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut dir = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
            dir.push("OpenDesk");
            dir
        })
}

/// `{data_dir}/opendesk.db`（background_job 等协调表）。
pub fn opendesk_db_path() -> PathBuf {
    opendesk_data_dir().join("opendesk.db")
}

/// `{data_dir}/crawler.db`（频道 / 关键词 / crawler settings）。
pub fn crawler_db_path() -> PathBuf {
    opendesk_data_dir().join("crawler.db")
}

/// `{data_dir}/knowledge.db`（知识库文档 / 分块 / 向量）。
pub fn knowledge_db_path() -> PathBuf {
    opendesk_data_dir().join("knowledge.db")
}

/// `{data_dir}/chat.db`（会话 / 消息 / 记忆）。
pub fn chat_db_path() -> PathBuf {
    opendesk_data_dir().join("chat.db")
}

/// `{data_dir}/.fastembed_cache`（本地嵌入模型缓存）。
pub fn embedding_cache_dir() -> PathBuf {
    opendesk_data_dir().join(".fastembed_cache")
}

/// `{data_dir}/opendesk-worker.lock`（worker 单实例文件锁）。
pub fn worker_lock_path() -> PathBuf {
    opendesk_data_dir().join("opendesk-worker.lock")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_under_data_dir() {
        let base = opendesk_data_dir();
        for path in [
            opendesk_db_path(),
            crawler_db_path(),
            knowledge_db_path(),
            chat_db_path(),
            embedding_cache_dir(),
            worker_lock_path(),
        ] {
            assert!(path.starts_with(&base), "{path:?} not under {base:?}");
        }
    }
}
