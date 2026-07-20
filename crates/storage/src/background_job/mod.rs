//! `background_job` persistence for `opendesk.db`.

mod sqlite;

pub use sqlite::SqliteBackgroundJobStore;
