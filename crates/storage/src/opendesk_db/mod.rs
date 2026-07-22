//! Shared SQLite connection + migrations for `opendesk.db`.
//!
//! 作者：coisini
//! 创建时间：2026-07-20

mod models;
pub mod schema;

use std::path::Path;
use std::sync::{Arc, Mutex};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use ports::repository::StoreError;

pub use models::{
    BackgroundJobRow, CustomerRow, MailAccountRow, MailMessageRow, MailTemplateRow,
    NewBackgroundJob, NewCustomerRow, NewMailAccountRow, NewMailMessageRow, NewMailTemplateRow,
    NewScriptSnippetRow, ScriptSnippetRow,
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations-opendesk");

/// Thread-safe handle to `opendesk.db` (main process + Worker share WAL mode).
#[derive(Clone)]
pub struct OpendeskDb {
    conn: Arc<Mutex<SqliteConnection>>,
}

impl OpendeskDb {
    /// Open or create `opendesk.db` and run pending migrations.
    ///
    /// # 参数
    /// - `path` — absolute or relative path to the database file
    ///
    /// # 返回值
    /// - `Ok(OpendeskDb)` — connection ready with WAL enabled
    /// - `Err(StoreError)` — migration or filesystem failure
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-20
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
        diesel::sql_query("PRAGMA foreign_keys = ON;")
            .execute(&mut conn)
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Run `f` with an exclusive lock on the underlying connection.
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-20
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
    fn open_runs_opendesk_migrations() {
        let dir = std::env::temp_dir().join(format!("opendesk_db_test_{}", Uuid::new_v4()));
        let path = dir.join("test.db");
        let db = OpendeskDb::open(&path).expect("open");
        db.with_conn(|conn| {
            diesel::sql_query("SELECT 1 FROM background_job LIMIT 1")
                .execute(conn)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(())
        })
        .expect("query");
        let _ = std::fs::remove_dir_all(dir);
    }
}
