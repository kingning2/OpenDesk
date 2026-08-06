use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDtoToolStatus {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub version: String,
    pub error: Option<String>,
}
