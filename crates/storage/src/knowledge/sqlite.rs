//! rusqlite-backed `KnowledgeStore` for the dedicated `knowledge.db` file.
//!
//! Mirrors `chat.db`'s pattern: a self-contained file with sqlite-vec registered
//! via auto-extension, since the `vec0` vtable does not fit Diesel's codegen model.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use ports::knowledge::{KnowledgeChunkHit, KnowledgeDocumentRecord, KnowledgeStore};
use ports::repository::StoreError;
use rusqlite::{params, Connection};
use uuid::Uuid;
use zerocopy::IntoBytes;

use crate::vec_extension::register_vec_extension;

/// bge-small-zh-v1.5 embedding dimension; must match `knowledge_chunk_vec`'s `float[512]`.
const VEC_DIMS: usize = 512;

/// Thread-safe handle to `knowledge.db`.
#[derive(Clone)]
pub struct SqliteKnowledgeStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteKnowledgeStore {
    /// Open or create `knowledge.db` and apply schema migrations.
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

    fn with_conn<F, T>(&self, f: F) -> Result<T, StoreError>
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

/// Apply schema migrations guarded by `PRAGMA user_version`.
fn migrate(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS knowledge_doc (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            source_type TEXT NOT NULL,
            status      TEXT NOT NULL,
            chunk_count INTEGER NOT NULL DEFAULT 0,
            created_at  INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS knowledge_chunk (
            id         TEXT PRIMARY KEY,
            doc_id     TEXT NOT NULL,
            content    TEXT NOT NULL,
            seq        INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (doc_id) REFERENCES knowledge_doc(id) ON DELETE CASCADE
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_chunk_vec USING vec0(
            embedding float[512] distance_metric=cosine
        );
        CREATE INDEX IF NOT EXISTS idx_knowledge_chunk_doc
            ON knowledge_chunk(doc_id, seq);",
    )
    .map_err(|error| StoreError::Unavailable(error.to_string()))?;
    Ok(())
}

fn document_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<KnowledgeDocumentRecord> {
    Ok(KnowledgeDocumentRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        source_type: row.get(2)?,
        status: row.get(3)?,
        chunk_count: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

impl KnowledgeStore for SqliteKnowledgeStore {
    fn create_document(
        &self,
        id: &str,
        name: &str,
        source_type: &str,
    ) -> Result<KnowledgeDocumentRecord, StoreError> {
        let now = now_millis();
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO knowledge_doc (id, name, source_type, status, chunk_count, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, 'parsing', 0, ?4, ?4)",
                params![id, name, source_type, now],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let record = conn
                .query_row(
                    "SELECT id, name, source_type, status, chunk_count, created_at, updated_at \
                     FROM knowledge_doc WHERE id = ?1",
                    [id],
                    document_from_row,
                )
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(record)
        })
    }

    fn insert_chunk(
        &self,
        doc_id: &str,
        content: &str,
        seq: i64,
        embedding: &[f32],
    ) -> Result<(), StoreError> {
        if embedding.len() != VEC_DIMS {
            return Err(StoreError::Unavailable(format!(
                "knowledge chunk embedding dim {} != expected {VEC_DIMS}",
                embedding.len()
            )));
        }
        self.with_conn(|conn| {
            let tx = conn
                .transaction()
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            tx.execute(
                "INSERT INTO knowledge_chunk (id, doc_id, content, seq, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    Uuid::new_v4().to_string(),
                    doc_id,
                    content,
                    seq,
                    now_millis()
                ],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let rowid = tx.last_insert_rowid();
            let embedding_bytes: &[u8] = embedding.as_bytes();
            tx.execute(
                "INSERT INTO knowledge_chunk_vec (rowid, embedding) VALUES (?1, ?2)",
                params![rowid, embedding_bytes],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            tx.commit()
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(())
        })
    }

    fn finish_document(&self, id: &str, chunk_count: i64) -> Result<(), StoreError> {
        let now = now_millis();
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE knowledge_doc SET status = 'ready', chunk_count = ?1, updated_at = ?2 \
                 WHERE id = ?3",
                params![chunk_count, now, id],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(())
        })
    }

    fn search_chunks(
        &self,
        query_embedding: &[f32],
        k: usize,
    ) -> Result<Vec<KnowledgeChunkHit>, StoreError> {
        if query_embedding.len() != VEC_DIMS {
            return Err(StoreError::Unavailable(format!(
                "query embedding dim {} != expected {VEC_DIMS}",
                query_embedding.len()
            )));
        }
        if k == 0 {
            return Ok(Vec::new());
        }
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT d.id, d.name, c.content, v.distance \
                     FROM knowledge_chunk_vec v \
                     JOIN knowledge_chunk c ON c.rowid = v.rowid \
                     JOIN knowledge_doc d ON d.id = c.doc_id \
                     WHERE v.embedding MATCH ?1 AND k = ?2 \
                     ORDER BY v.distance ASC",
                )
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let query_bytes: &[u8] = query_embedding.as_bytes();
            let rows = stmt
                .query_map(params![query_bytes, k as i64], |row| {
                    Ok(KnowledgeChunkHit {
                        doc_id: row.get(0)?,
                        name: row.get(1)?,
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

    fn list_documents(&self) -> Result<Vec<KnowledgeDocumentRecord>, StoreError> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, source_type, status, chunk_count, created_at, updated_at \
                     FROM knowledge_doc ORDER BY updated_at DESC",
                )
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let rows = stmt
                .query_map([], document_from_row)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let mut documents = Vec::new();
            for row in rows {
                documents.push(row.map_err(|error| StoreError::Unavailable(error.to_string()))?);
            }
            Ok(documents)
        })
    }

    fn delete_document(&self, id: &str) -> Result<(), StoreError> {
        self.with_conn(|conn| {
            // Vector rows have no FK to knowledge_chunk, so remove them first by rowid.
            conn.execute(
                "DELETE FROM knowledge_chunk_vec WHERE rowid IN \
                 (SELECT rowid FROM knowledge_chunk WHERE doc_id = ?1)",
                [id],
            )
            .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            let deleted = conn
                .execute("DELETE FROM knowledge_doc WHERE id = ?1", [id])
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            if deleted == 0 {
                return Err(StoreError::NotFound);
            }
            Ok(())
        })
    }

    fn count_documents(&self) -> Result<usize, StoreError> {
        self.with_conn(|conn| {
            conn.query_row("SELECT COUNT(*) FROM knowledge_doc", [], |row| row.get(0))
                .map_err(|error| StoreError::Unavailable(error.to_string()))
        })
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一个临时 knowledge.db 的 store；返回的 TempDir 必须存活到 store 关闭。
    fn test_store() -> (SqliteKnowledgeStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("knowledge.db");
        let store = SqliteKnowledgeStore::open(&path).expect("open knowledge store");
        (store, dir)
    }

    #[test]
    fn create_insert_search_and_delete_roundtrip() {
        let (store, _dir) = test_store();
        let embedding = vec![0.25f32; VEC_DIMS];

        let doc = store
            .create_document("doc-1", "产品手册.md", "md")
            .expect("create document");
        assert_eq!(doc.status, "parsing");
        assert_eq!(store.count_documents().expect("count"), 1);

        store
            .insert_chunk("doc-1", "产品支持多语言邮件模板。", 0, &embedding)
            .expect("insert chunk");
        store.finish_document("doc-1", 1).expect("finish document");

        let documents = store.list_documents().expect("list documents");
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].status, "ready");
        assert_eq!(documents[0].chunk_count, 1);

        // 检索应命中插入的分块。
        let hits = store.search_chunks(&embedding, 5).expect("search chunks");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].doc_id, "doc-1");
        assert!(hits[0].content.contains("多语言"));

        // 删除后检索为空。
        store.delete_document("doc-1").expect("delete document");
        assert_eq!(store.count_documents().expect("count"), 0);
        let hits = store.search_chunks(&embedding, 5).expect("search chunks");
        assert!(hits.is_empty());
    }

    #[test]
    fn rejects_wrong_embedding_dim() {
        let (store, _dir) = test_store();
        store
            .create_document("doc-2", "a.txt", "txt")
            .expect("create document");
        let bad = vec![0.1f32; 100];
        let result = store.insert_chunk("doc-2", "内容", 0, &bad);
        assert!(result.is_err(), "wrong dim must be rejected");
    }
}
