//! 持久化端口共享错误。

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("not found")]
    NotFound,
    #[error("store unavailable: {0}")]
    Unavailable(String),
}
