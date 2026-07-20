//! Shared SQLite connection + migrations for crawler storage.

mod models;
pub mod schema;

use std::path::Path;
use std::sync::{Arc, Mutex};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use ports::repository::StoreError;

pub use models::{CrawlerChannelRow, NewCrawlerChannel, NewCrawlerKeyword};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Thread-safe handle to `crawler.db` (keywords + channels share one connection pool slot).
#[derive(Clone)]
pub struct CrawlerDb {
    conn: Arc<Mutex<SqliteConnection>>,
}

impl CrawlerDb {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
        }

        let database_url = sqlite_url(path.as_ref());
        let mut conn = SqliteConnection::establish(&database_url)
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
        diesel::sql_query("PRAGMA journal_mode = WAL;")
            .execute(&mut conn)
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn with_conn<F, T>(&self, f: F) -> Result<T, StoreError>
    where
        F: FnOnce(&mut SqliteConnection) -> Result<T, StoreError>,
    {
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
        f(&mut conn)
    }
}

fn sqlite_url(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    if normalized.starts_with('/') || normalized.contains(':') {
        format!("sqlite:///{normalized}")
    } else {
        format!("sqlite://{normalized}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn open_runs_migrations() {
        let dir = std::env::temp_dir().join(format!("crawler_db_test_{}", Uuid::new_v4()));
        let path = dir.join("test.db");
        let db = CrawlerDb::open(&path).expect("open");
        db.with_conn(|conn| {
            diesel::sql_query("SELECT 1 FROM crawler_keyword LIMIT 1")
                .execute(conn)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(())
        })
        .expect("query");
        let _ = std::fs::remove_dir_all(dir);
    }
}
