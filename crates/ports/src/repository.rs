//! Persistence repository traits (skeleton).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("not found")]
    NotFound,
    #[error("store unavailable: {0}")]
    Unavailable(String),
}

/// Generic record store port — implementations live in `storage`.
pub trait RecordStore: Send + Sync {
    fn ready(&self) -> bool;
    fn get(&self, table: &str, id: &str) -> Result<Option<Vec<u8>>, StoreError>;
    fn put(&self, table: &str, id: &str, data: &[u8]) -> Result<(), StoreError>;
    fn delete(&self, table: &str, id: &str) -> Result<bool, StoreError>;
}
