//! Handle `knowledge_import` background jobs: parse + vectorize a local document.
//!
//! The file path arrives in the job payload (selected by the user via the Tauri
//! dialog in the main process). The worker reads the file bytes, then runs the
//! knowledge import use-case (parse → chunk → embed → store).

use std::sync::Arc;

use agent::embedding::Embedder;
use knowledge::ImportDocument;
use ports::background_job::{BackgroundJobRecord, KnowledgeImportPayload};
use ports::knowledge::KnowledgeStore;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("invalid payload: {0}")]
    InvalidPayload(String),
    #[error("read file failed: {0}")]
    ReadFailed(String),
    #[error("import failed: {0}")]
    Import(String),
}

/// Execute one knowledge import job.
///
/// `store` and `embedder` are shared across jobs (constructed once in `main.rs`).
pub async fn handle(
    job: &BackgroundJobRecord,
    store: Arc<dyn KnowledgeStore>,
    embedder: Arc<dyn Embedder>,
) -> Result<(), HandlerError> {
    let payload: KnowledgeImportPayload = serde_json::from_str(&job.payload_json)
        .map_err(|error| HandlerError::InvalidPayload(error.to_string()))?;

    let file_path = std::path::PathBuf::from(&payload.file_path);
    if !file_path.is_file() {
        return Err(HandlerError::ReadFailed(format!(
            "文件不存在: {}",
            payload.file_path
        )));
    }
    let bytes =
        std::fs::read(&file_path).map_err(|error| HandlerError::ReadFailed(error.to_string()))?;
    let name = file_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("document")
        .to_string();

    ImportDocument::execute(&name, &bytes, store, embedder)
        .await
        .map_err(HandlerError::Import)?;

    tracing::info!(job_id = %job.id, file_path = %payload.file_path, "knowledge import completed");
    Ok(())
}
