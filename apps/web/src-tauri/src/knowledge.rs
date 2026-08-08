//! 知识库 RPC helper：文档导入 / 列表 / 删除 + 工具状态。

use app_core::AppState;
use common::contracts::{
    KnowledgeDtoDocument, KnowledgeDtoToolStatus, KnowledgeIpcDocumentDeleteRequest,
    KnowledgeIpcDocumentImportRequest, KnowledgeIpcDocumentImportResponse,
    KnowledgeIpcDocumentListResponse, KnowledgeIpcToolStatusResponse,
};
use common::tools::{detect_tool, ToolId, ToolStatus};
use knowledge::{DeleteDocument, ListDocuments};
use ports::background_job::JOB_TYPE_KNOWLEDGE_IMPORT;
use serde_json::{json, Value};

fn record_to_dto(record: ports::knowledge::KnowledgeDocumentRecord) -> KnowledgeDtoDocument {
    KnowledgeDtoDocument {
        id: record.id,
        name: record.name,
        source_type: record.source_type,
        status: record.status,
        chunk_count: record.chunk_count,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn tool_status_to_dto(status: ToolStatus) -> KnowledgeDtoToolStatus {
    KnowledgeDtoToolStatus {
        id: status.id.to_string(),
        name: status.name.to_string(),
        installed: status.installed,
        version: status.version,
        error: status.error,
    }
}

/// 入队知识库导入 job。
pub async fn doc_import(
    app: &AppState,
    req: KnowledgeIpcDocumentImportRequest,
) -> Result<Value, String> {
    let file_path = req.file_path;
    let payload = json!({ "file_path": file_path }).to_string();
    let job_store = app.job_store.clone();
    let job_id =
        tokio::task::spawn_blocking(move || job_store.enqueue(JOB_TYPE_KNOWLEDGE_IMPORT, &payload))
            .await
            .map_err(|error| error.to_string())?
            .map_err(|error| error.to_string())?;
    tracing::info!(job_id = %job_id, "knowledge import job enqueued");
    Ok(json!(KnowledgeIpcDocumentImportResponse {
        ok: true,
        job_id: Some(job_id),
        error_message: None,
    }))
}

/// 列出知识库文档。
pub async fn doc_list(app: &AppState) -> Result<Value, String> {
    let store = app.knowledge_store.clone();
    let documents = tokio::task::spawn_blocking(move || ListDocuments::execute(store))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(record_to_dto)
        .collect::<Vec<_>>();
    Ok(json!(KnowledgeIpcDocumentListResponse {
        documents_json: json!(documents).to_string(),
    }))
}

/// 删除文档。
pub async fn doc_delete(
    app: &AppState,
    req: KnowledgeIpcDocumentDeleteRequest,
) -> Result<Value, String> {
    let store = app.knowledge_store.clone();
    let document_id = req.document_id;
    let ok = tokio::task::spawn_blocking(move || DeleteDocument::execute(store, &document_id))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    Ok(json!({ "ok": ok }))
}

/// 查询解析工具状态。
pub async fn tool_status() -> Result<Value, String> {
    let tools = [ToolId::Pandoc, ToolId::Tesseract, ToolId::Pdfium]
        .into_iter()
        .map(|id| tool_status_to_dto(detect_tool(id)))
        .collect::<Vec<_>>();
    Ok(json!(KnowledgeIpcToolStatusResponse {
        tools_json: json!(tools).to_string(),
    }))
}
