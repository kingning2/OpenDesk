//! rusqlite-backed `ChatStore` for the dedicated `chat.db` file.
//!
//! chat.db is intentionally separate from Diesel-managed opendesk.db: sqlite-vec
//! (vector search, added in phase 2) must be registered on the raw connection,
//! which does not fit Diesel's schema/codegen model. Everything chat-related
//! (sessions, messages, memory) lives in this one self-contained file.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use ports::chat::{
    ChatMemoryStore, ChatMessageRecord, ChatSessionRecord, ChatStore, MemoryHit, MemoryRecord,
    SaveChatMessage,
};
use ports::repository::StoreError;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use zerocopy::IntoBytes;

use crate::vec_extension::register_vec_extension;

/// Current `PRAGMA user_version`; bump + extend `migrate` when the schema changes.
const SCHEMA_VERSION: i64 = 2;

/// bge-small-zh-v1.5 embedding dimension; must match `chat_memory_vec`'s `float[512]`.
const MEMORY_VEC_DIMS: usize = 512;

/// Thread-safe handle to `chat.db`.
#[derive(Clone)]
pub struct SqliteChatStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteChatStore {
    /// Open or create `chat.db` and apply schema migrations.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
        }
        register_vec_extension();
        let conn =
            Connection::open(path).map_err(|error| StoreError::Unavailable(error.to_string()))?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
        migrate(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Run `f` with an exclusive lock on the underlying connection.
    pub fn with_conn<F, T>(&self, f: F) -> Result<T, StoreError>
    where
        F: FnOnce(&mut Connection) -> Result<T, StoreError>,
    {
        let mut conn = self
            .conn
            .lock()
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
        f(&mut conn)
    }
}

/// Apply pending schema migrations guarded by `PRAGMA user_version`.
fn migrate(conn: &Connection) -> Result<(), StoreError> {
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| StoreError::Unavailable(error.to_string()))?;
    if version < 1 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS chat_session (
                id               TEXT PRIMARY KEY,
                title            TEXT NOT NULL DEFAULT '',
                created_at       INTEGER NOT NULL,
                updated_at       INTEGER NOT NULL,
                last_message_at  INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS chat_message (
                id         TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role       TEXT NOT NULL,
                content    TEXT NOT NULL DEFAULT '',
                thinking   TEXT NOT NULL DEFAULT '',
                tools_json TEXT NOT NULL DEFAULT '',
                seq        INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES chat_session(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_chat_message_session
                ON chat_message(session_id, seq);
            PRAGMA user_version = 1;",
        )
        .map_err(|error| StoreError::Unavailable(error.to_string()))?;
    }
    if version < SCHEMA_VERSION {
        // sqlite-vec is registered via auto-extension before `Connection::open`,
        // so the `vec0` vtable is available here.
        conn.execute_batch(
            "ALTER TABLE chat_session ADD COLUMN summary_state_json TEXT NOT NULL DEFAULT '';
            CREATE TABLE IF NOT EXISTS chat_memory (
                id                TEXT PRIMARY KEY,
                kind              TEXT NOT NULL,
                content           TEXT NOT NULL,
                source_session_id TEXT,
                created_at        INTEGER NOT NULL
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS chat_memory_vec USING vec0(
                embedding float[512] distance_metric=cosine
            );
            CREATE INDEX IF NOT EXISTS idx_chat_memory_session
                ON chat_memory(source_session_id);
            PRAGMA user_version = 2;",
        )
        .map_err(|error| StoreError::Unavailable(error.to_string()))?;
    }
    Ok(())
}

fn session_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChatSessionRecord> {
    Ok(ChatSessionRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
        last_message_at: row.get(4)?,
        message_count: row.get(5)?,
    })
}

const SESSION_SELECT: &str = "SELECT s.id, s.title, s.created_at, s.updated_at, \
     s.last_message_at, \
     (SELECT COUNT(*) FROM chat_message m WHERE m.session_id = s.id) \
     FROM chat_session s";

/// Query one session on an already-locked connection (avoids re-entrant locking).
fn select_session(
    conn: &mut Connection,
    id: &str,
) -> Result<Option<ChatSessionRecord>, StoreError> {
    conn.query_row(
        &format!("{SESSION_SELECT} WHERE s.id = ?1"),
        [id],
        session_from_row,
    )
    .optional()
    .map_err(|error| StoreError::Unavailable(error.to_string()))
}

impl ChatStore for SqliteChatStore {
    fn list_sessions(&self) -> Result<Vec<ChatSessionRecord>, StoreError> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(&format!("{SESSION_SELECT} ORDER BY s.updated_at DESC"))
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let rows = stmt
                .query_map([], session_from_row)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let mut sessions = Vec::new();
            for row in rows {
                sessions.push(row.map_err(|error| StoreError::Unavailable(error.to_string()))?);
            }
            Ok(sessions)
        })
    }

    fn get_session(&self, id: &str) -> Result<Option<ChatSessionRecord>, StoreError> {
        self.with_conn(|conn| select_session(conn, id))
    }

    fn create_session(&self, id: &str, title: &str) -> Result<ChatSessionRecord, StoreError> {
        let now = now_millis();
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO chat_session (id, title, created_at, updated_at, last_message_at) \
                 VALUES (?1, ?2, ?3, ?3, 0)",
                params![id, title, now],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            select_session(conn, id)?.ok_or(StoreError::NotFound)
        })
    }

    fn rename_session(&self, id: &str, title: &str) -> Result<ChatSessionRecord, StoreError> {
        let now = now_millis();
        self.with_conn(|conn| {
            let updated = conn
                .execute(
                    "UPDATE chat_session SET title = ?1, updated_at = ?2 WHERE id = ?3",
                    params![title, now, id],
                )
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            if updated == 0 {
                return Err(StoreError::NotFound);
            }
            select_session(conn, id)?.ok_or(StoreError::NotFound)
        })
    }

    fn delete_session(&self, id: &str) -> Result<(), StoreError> {
        self.with_conn(|conn| {
            // Vector rows have no FK to chat_memory, so remove them first by rowid.
            conn.execute(
                "DELETE FROM chat_memory_vec WHERE rowid IN \
                 (SELECT rowid FROM chat_memory WHERE source_session_id = ?1)",
                [id],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            conn.execute("DELETE FROM chat_memory WHERE source_session_id = ?1", [id])
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            conn.execute("DELETE FROM chat_session WHERE id = ?1", [id])
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(())
        })
    }

    fn load_messages(&self, session_id: &str) -> Result<Vec<ChatMessageRecord>, StoreError> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, session_id, role, content, thinking, tools_json, seq, created_at \
                     FROM chat_message WHERE session_id = ?1 ORDER BY seq ASC",
                )
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let rows = stmt
                .query_map([session_id], |row| {
                    Ok(ChatMessageRecord {
                        id: row.get(0)?,
                        session_id: row.get(1)?,
                        role: row.get(2)?,
                        content: row.get(3)?,
                        thinking: non_empty(row.get(4)?),
                        tools_json: non_empty(row.get(5)?),
                        seq: row.get(6)?,
                        created_at: row.get(7)?,
                    })
                })
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let mut messages = Vec::new();
            for row in rows {
                messages.push(row.map_err(|error| StoreError::Unavailable(error.to_string()))?);
            }
            Ok(messages)
        })
    }

    fn save_message(&self, input: SaveChatMessage) -> Result<ChatMessageRecord, StoreError> {
        let now = now_millis();
        self.with_conn(|conn| {
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM chat_session WHERE id = ?1)",
                    [&input.session_id],
                    |row| row.get(0),
                )
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            if !exists {
                return Err(StoreError::NotFound);
            }
            let tx = conn
                .transaction()
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let seq: i64 = tx
                .query_row(
                    "SELECT COALESCE(MAX(seq), 0) + 1 FROM chat_message WHERE session_id = ?1",
                    [&input.session_id],
                    |row| row.get(0),
                )
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            tx.execute(
                "INSERT OR IGNORE INTO chat_message \
                 (id, session_id, role, content, thinking, tools_json, seq, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    input.id,
                    input.session_id,
                    input.role,
                    input.content,
                    input.thinking.as_deref().unwrap_or(""),
                    input.tools_json.as_deref().unwrap_or(""),
                    seq,
                    now,
                ],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            tx.execute(
                "UPDATE chat_session SET updated_at = ?1, last_message_at = ?1 WHERE id = ?2",
                params![now, input.session_id],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            // Auto-title an unset session from its first user message.
            if input.role == "user" {
                let title: String = tx
                    .query_row(
                        "SELECT title FROM chat_session WHERE id = ?1",
                        [&input.session_id],
                        |row| row.get(0),
                    )
                    .map_err(|error| StoreError::Unavailable(error.to_string()))?;
                if title.is_empty() {
                    let auto = input.content.chars().take(20).collect::<String>();
                    tx.execute(
                        "UPDATE chat_session SET title = ?1 WHERE id = ?2",
                        params![auto, input.session_id],
                    )
                    .map_err(|error| StoreError::Unavailable(error.to_string()))?;
                }
            }
            tx.commit()
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(ChatMessageRecord {
                id: input.id,
                session_id: input.session_id,
                role: input.role,
                content: input.content,
                thinking: input.thinking,
                tools_json: input.tools_json,
                seq,
                created_at: now,
            })
        })
    }

    fn get_summary_state(&self, session_id: &str) -> Result<Option<String>, StoreError> {
        self.with_conn(|conn| {
            let value = conn
                .query_row(
                    "SELECT summary_state_json FROM chat_session WHERE id = ?1",
                    [session_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(value.filter(|text| !text.is_empty()))
        })
    }

    fn set_summary_state(&self, session_id: &str, json: &str) -> Result<(), StoreError> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE chat_session SET summary_state_json = ?1 WHERE id = ?2",
                params![json, session_id],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(())
        })
    }
}

impl ChatMemoryStore for SqliteChatStore {
    fn insert_memory(
        &self,
        kind: &str,
        content: &str,
        source_session_id: Option<&str>,
        embedding: &[f32],
    ) -> Result<MemoryRecord, StoreError> {
        if embedding.len() != MEMORY_VEC_DIMS {
            return Err(StoreError::Unavailable(format!(
                "memory embedding dim {} != expected {MEMORY_VEC_DIMS}",
                embedding.len()
            )));
        }
        let id = Uuid::new_v4().to_string();
        let created_at = now_millis();
        self.with_conn(|conn| {
            let tx = conn
                .transaction()
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            tx.execute(
                "INSERT INTO chat_memory (id, kind, content, source_session_id, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, kind, content, source_session_id, created_at],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let rowid = tx.last_insert_rowid();
            let embedding_bytes: &[u8] = embedding.as_bytes();
            tx.execute(
                "INSERT INTO chat_memory_vec (rowid, embedding) VALUES (?1, ?2)",
                params![rowid, embedding_bytes],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            tx.commit()
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(MemoryRecord {
                id,
                kind: kind.to_string(),
                content: content.to_string(),
                source_session_id: source_session_id.map(str::to_string),
                created_at,
            })
        })
    }

    fn search_memories(
        &self,
        query_embedding: &[f32],
        k: usize,
    ) -> Result<Vec<MemoryHit>, StoreError> {
        if query_embedding.len() != MEMORY_VEC_DIMS {
            return Err(StoreError::Unavailable(format!(
                "query embedding dim {} != expected {MEMORY_VEC_DIMS}",
                query_embedding.len()
            )));
        }
        if k == 0 {
            return Ok(Vec::new());
        }
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT m.rowid, m.kind, m.content, v.distance \
                     FROM chat_memory_vec v \
                     JOIN chat_memory m ON m.rowid = v.rowid \
                     WHERE v.embedding MATCH ?1 AND k = ?2 \
                     ORDER BY v.distance ASC",
                )
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let query_bytes: &[u8] = query_embedding.as_bytes();
            let rows = stmt
                .query_map(params![query_bytes, k as i64], |row| {
                    Ok(MemoryHit {
                        rowid: row.get(0)?,
                        kind: row.get(1)?,
                        content: row.get(2)?,
                        distance: row.get(3)?,
                    })
                })
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let mut hits = Vec::new();
            for row in rows {
                hits.push(row.map_err(|error| StoreError::Unavailable(error.to_string()))?);
            }
            Ok(hits)
        })
    }

    fn latest_session_digest(&self, session_id: &str) -> Result<Option<String>, StoreError> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT content FROM chat_memory \
                 WHERE kind = 'digest' AND source_session_id = ?1 \
                 ORDER BY created_at DESC LIMIT 1",
                [session_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| StoreError::Unavailable(error.to_string()))
        })
    }

    fn delete_session_memories(&self, session_id: &str) -> Result<(), StoreError> {
        self.with_conn(|conn| {
            conn.execute(
                "DELETE FROM chat_memory_vec WHERE rowid IN \
                 (SELECT rowid FROM chat_memory WHERE source_session_id = ?1)",
                [session_id],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            conn.execute(
                "DELETE FROM chat_memory WHERE source_session_id = ?1",
                [session_id],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(())
        })
    }
}

/// Treat an empty stored string as `None`.
fn non_empty(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as i64)
        .unwrap_or(0)
}
