//! SQLite-backed record store (skeleton).

use std::collections::HashMap;
use std::sync::RwLock;

use ports::repository::{RecordStore, StoreError};

#[derive(Default)]
struct MemoryTable {
    rows: HashMap<String, Vec<u8>>,
}

pub struct SqliteRecordStore {
    tables: RwLock<HashMap<String, MemoryTable>>,
}

impl SqliteRecordStore {
    pub fn new() -> Self {
        Self {
            tables: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for SqliteRecordStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordStore for SqliteRecordStore {
    fn ready(&self) -> bool {
        self.tables.read().is_ok()
    }

    fn get(&self, table: &str, id: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let tables = self
            .tables
            .read()
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
        Ok(tables
            .get(table)
            .and_then(|entry| entry.rows.get(id))
            .cloned())
    }

    fn put(&self, table: &str, id: &str, data: &[u8]) -> Result<(), StoreError> {
        let mut tables = self
            .tables
            .write()
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
        tables
            .entry(table.to_string())
            .or_default()
            .rows
            .insert(id.to_string(), data.to_vec());
        Ok(())
    }

    fn delete(&self, table: &str, id: &str) -> Result<bool, StoreError> {
        let mut tables = self
            .tables
            .write()
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
        Ok(tables
            .get_mut(table)
            .and_then(|entry| entry.rows.remove(id))
            .is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_delete_roundtrip() {
        let store = SqliteRecordStore::new();
        assert!(store.ready());
        store.put("demo", "1", b"hello").ok();
        let value = store.get("demo", "1").ok().flatten();
        assert_eq!(value.as_deref(), Some(b"hello".as_slice()));
        assert_eq!(store.delete("demo", "1").ok(), Some(true));
        assert!(store.get("demo", "1").ok().flatten().is_none());
    }
}
