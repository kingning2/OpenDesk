//! Chat persistence adapter for local `chat.db`.

pub mod sqlite;

pub use sqlite::SqliteChatStore;
