use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDtoDocument {
    pub id: String,
    pub name: String,
    pub source_type: String,
    pub status: String,
    pub chunk_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}
