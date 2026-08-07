//! Worker 数据库路径，统一委托 `common::tools::paths`。
//!
//! 作者：coisini
//! 创建时间：2026-07-20

pub use common::tools::paths::{
    crawler_db_path, embedding_cache_dir, knowledge_db_path, opendesk_db_path, worker_lock_path,
};
