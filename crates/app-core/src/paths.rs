//! 本地 SQLite 数据文件路径，统一委托 `common::tools::paths`。

pub use common::tools::paths::{
    chat_db_path, crawler_db_path, embedding_cache_dir, knowledge_db_path, opendesk_db_path,
};
