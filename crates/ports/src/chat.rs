//! Chat session + message persistence port.

use crate::repository::StoreError;

/// A persisted chat session (conversation).
#[derive(Debug, Clone)]
pub struct ChatSessionRecord {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_message_at: i64,
    pub message_count: i64,
}

/// A completed chat message (placeholders / streaming are not persisted).
#[derive(Debug, Clone)]
pub struct ChatMessageRecord {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub thinking: Option<String>,
    pub tools_json: Option<String>,
    pub seq: i64,
    pub created_at: i64,
}

/// Input to persist one completed message; `seq` and `created_at` are assigned by the store.
#[derive(Debug, Clone)]
pub struct SaveChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub thinking: Option<String>,
    pub tools_json: Option<String>,
}

/// Chat session + message persistence port (backed by local `chat.db`).
pub trait ChatStore: Send + Sync {
    /// List sessions, newest updated first, with message counts.
    fn list_sessions(&self) -> Result<Vec<ChatSessionRecord>, StoreError>;

    /// Fetch one session by id.
    fn get_session(&self, id: &str) -> Result<Option<ChatSessionRecord>, StoreError>;

    /// Create a session; an empty `title` means "unset" (auto-titled on first message).
    fn create_session(&self, id: &str, title: &str) -> Result<ChatSessionRecord, StoreError>;

    /// Rename a session.
    fn rename_session(&self, id: &str, title: &str) -> Result<ChatSessionRecord, StoreError>;

    /// Delete a session and cascade its messages.
    fn delete_session(&self, id: &str) -> Result<(), StoreError>;

    /// Load completed messages ordered by `seq`.
    fn load_messages(&self, session_id: &str) -> Result<Vec<ChatMessageRecord>, StoreError>;

    /// Persist one completed message; assigns `seq` + `created_at` and touches the session.
    /// An empty-titled session is auto-titled from the first user message.
    fn save_message(&self, input: SaveChatMessage) -> Result<ChatMessageRecord, StoreError>;

    /// Read a session's digest bookkeeping JSON (`{"last_digest_seq": N, ...}`), if any.
    fn get_summary_state(&self, session_id: &str) -> Result<Option<String>, StoreError>;

    /// Persist a session's digest bookkeeping JSON.
    fn set_summary_state(&self, session_id: &str, json: &str) -> Result<(), StoreError>;
}

/// A persisted long-term memory entry (digest / fact) with its embedding.
#[derive(Debug, Clone)]
pub struct MemoryRecord {
    pub id: String,
    pub kind: String,
    pub content: String,
    pub source_session_id: Option<String>,
    pub created_at: i64,
}

/// One nearest-neighbor memory hit from vector search.
#[derive(Debug, Clone)]
pub struct MemoryHit {
    pub rowid: i64,
    pub kind: String,
    pub content: String,
    pub distance: f32,
}

/// Long-term memory persistence + vector search port (chat.db, sqlite-vec).
pub trait ChatMemoryStore: Send + Sync {
    /// Persist one memory with its embedding (vector row joined by implicit rowid).
    fn insert_memory(
        &self,
        kind: &str,
        content: &str,
        source_session_id: Option<&str>,
        embedding: &[f32],
    ) -> Result<MemoryRecord, StoreError>;

    /// KNN-search the top-k memories nearest to `query_embedding`, ascending distance.
    fn search_memories(
        &self,
        query_embedding: &[f32],
        k: usize,
    ) -> Result<Vec<MemoryHit>, StoreError>;

    /// Latest digest memory for a session (used for window compression), if any.
    fn latest_session_digest(&self, session_id: &str) -> Result<Option<String>, StoreError>;

    /// Delete all memories originating from a session (session deletion cascade).
    fn delete_session_memories(&self, session_id: &str) -> Result<(), StoreError>;
}
