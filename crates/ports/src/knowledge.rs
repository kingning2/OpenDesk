//! Knowledge base document persistence + vector search port.

use crate::repository::StoreError;

/// One imported knowledge document (file parsed + vectorized).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeDocumentRecord {
    pub id: String,
    pub name: String,
    pub source_type: String,
    pub status: String,
    pub chunk_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// One nearest-neighbor chunk hit from vector search.
#[derive(Debug, Clone)]
pub struct KnowledgeChunkHit {
    pub doc_id: String,
    pub name: String,
    pub content: String,
    pub distance: f32,
}

/// Knowledge document persistence + vector search port (backed by `knowledge.db`, sqlite-vec).
///
/// `insert_chunk` takes an already-computed embedding; the caller embeds via
/// `agent::embedding::Embedder` (512-dim bge-small-zh-v1.5).
pub trait KnowledgeStore: Send + Sync {
    /// Insert a document record; returns the persisted record.
    fn create_document(
        &self,
        id: &str,
        name: &str,
        source_type: &str,
    ) -> Result<KnowledgeDocumentRecord, StoreError>;

    /// Insert one vectorized chunk belonging to a document.
    fn insert_chunk(
        &self,
        doc_id: &str,
        content: &str,
        seq: i64,
        embedding: &[f32],
    ) -> Result<(), StoreError>;

    /// Update a document's status / chunk count after import finishes.
    fn finish_document(&self, id: &str, chunk_count: i64) -> Result<(), StoreError>;

    /// KNN-search the top-k chunks nearest to `query_embedding`, ascending distance.
    fn search_chunks(
        &self,
        query_embedding: &[f32],
        k: usize,
    ) -> Result<Vec<KnowledgeChunkHit>, StoreError>;

    /// List documents, newest updated first.
    fn list_documents(&self) -> Result<Vec<KnowledgeDocumentRecord>, StoreError>;

    /// Delete a document and cascade its chunks.
    fn delete_document(&self, id: &str) -> Result<(), StoreError>;

    /// Total number of documents (used to skip retrieval when the KB is empty).
    fn count_documents(&self) -> Result<usize, StoreError>;
}
