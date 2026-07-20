//! Diesel-backed `crawler_setting` store.

mod sqlite;

pub use sqlite::SqliteCrawlerSettingsStore;
