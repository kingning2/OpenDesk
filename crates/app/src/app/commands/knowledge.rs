//! 知识库 Tauri command：文档导入 / 列表 / 删除 + 解析工具状态 / 下载。

use common::contracts::{
    KnowledgeDtoDocument, KnowledgeDtoToolStatus, KnowledgeEventDownloadProgress,
    KnowledgeIpcDocumentDeleteRequest, KnowledgeIpcDocumentDeleteResponse,
    KnowledgeIpcDocumentImportRequest, KnowledgeIpcDocumentImportResponse,
    KnowledgeIpcDocumentListResponse, KnowledgeIpcToolDownloadRequest,
    KnowledgeIpcToolDownloadResponse, KnowledgeIpcToolStatusResponse,
};
use knowledge::{DeleteDocument, ListDocuments, ToolId};
use ports::background_job::JOB_TYPE_KNOWLEDGE_IMPORT;
use serde_json::json;
use tauri::Emitter;

use crate::app::state::AppState;

/// 解析工具进度事件 topic（与前端监听对齐）。
const TOOL_PROGRESS_EVENT: &str = "knowledge:tool/progress";

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

fn tool_status_to_dto(status: knowledge::ToolStatus) -> KnowledgeDtoToolStatus {
    KnowledgeDtoToolStatus {
        id: status.id.to_string(),
        name: status.name.to_string(),
        installed: status.installed,
        version: status.version,
        error: status.error,
    }
}

/// 入队一个知识库导入 job（文件由前端 Tauri dialog 选择，worker 按路径读文件解析）。
///
/// 解析 / 向量化在 `opendesk-worker` 进程执行；本命令只入队并立即返回 job id。
/// 导入完成由主进程轮询 `knowledge_doc` 状态变化后推 `knowledge:import/updated` 事件。
///
/// # 参数
/// - `state` — 应用共享状态
/// - `request` — 本地文件绝对路径
///
/// # 返回值
/// 入队成功返回 `ok=true` + job_id；入队失败返回错误描述。
#[tauri::command]
pub async fn knowledge_doc_import(
    state: tauri::State<'_, AppState>,
    request: KnowledgeIpcDocumentImportRequest,
) -> Result<KnowledgeIpcDocumentImportResponse, String> {
    let file_path = request.file_path;
    let payload = json!({ "file_path": file_path }).to_string();
    let job_store = state.job_store.clone();
    let job_id = tauri::async_runtime::spawn_blocking(move || {
        job_store.enqueue(JOB_TYPE_KNOWLEDGE_IMPORT, &payload)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;
    tracing::info!(job_id = %job_id, "knowledge import job enqueued");
    Ok(KnowledgeIpcDocumentImportResponse {
        ok: true,
        job_id: Some(job_id),
        error_message: None,
    })
}

/// 列出知识库所有文档（最新更新在前）。
#[tauri::command]
pub async fn knowledge_doc_list(
    state: tauri::State<'_, AppState>,
) -> Result<KnowledgeIpcDocumentListResponse, String> {
    let store = state.knowledge_store.clone();
    let documents = tauri::async_runtime::spawn_blocking(move || ListDocuments::execute(store))
        .await
        .map_err(|error| error.to_string())??
        .into_iter()
        .map(record_to_dto)
        .collect::<Vec<_>>();
    Ok(KnowledgeIpcDocumentListResponse {
        documents_json: json!(documents).to_string(),
    })
}

/// 删除一个知识库文档（级联删除分块与向量）。
#[tauri::command]
pub async fn knowledge_doc_delete(
    state: tauri::State<'_, AppState>,
    request: KnowledgeIpcDocumentDeleteRequest,
) -> Result<KnowledgeIpcDocumentDeleteResponse, String> {
    let store = state.knowledge_store.clone();
    let document_id = request.document_id;
    let ok =
        tauri::async_runtime::spawn_blocking(move || DeleteDocument::execute(store, &document_id))
            .await
            .map_err(|error| error.to_string())??;
    Ok(KnowledgeIpcDocumentDeleteResponse { ok })
}

/// 查询三个解析工具的安装状态。
#[tauri::command]
pub async fn knowledge_tool_status(
    state: tauri::State<'_, AppState>,
) -> Result<KnowledgeIpcToolStatusResponse, String> {
    let _ = state;
    let tools = [ToolId::Pandoc, ToolId::Tesseract, ToolId::Pdfium]
        .into_iter()
        .map(|id| tool_status_to_dto(knowledge::detect_tool(id)))
        .collect::<Vec<_>>();
    Ok(KnowledgeIpcToolStatusResponse {
        tools_json: json!(tools).to_string(),
    })
}

/// 下载并安装一个解析工具；进度经 `knowledge:tool/progress` 事件推送。
///
/// # 参数
/// - `app` — Tauri app handle（用于事件推送）
/// - `state` — 应用共享状态
/// - `request` — 工具 id（pandoc / tesseract / pdfium）
///
/// # 返回值
/// 任务已启动返回 `ok=true`；工具 id 非法或任务无法启动返回错误。
#[tauri::command]
pub async fn knowledge_tool_download(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    request: KnowledgeIpcToolDownloadRequest,
) -> Result<KnowledgeIpcToolDownloadResponse, String> {
    let _ = state;
    let tool = ToolId::parse(&request.tool).ok_or_else(|| format!("未知工具: {}", request.tool))?;
    let handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let handle_for_emit = handle.clone();
        let emit = move |progress: knowledge::DownloadProgress| {
            let _ = handle_for_emit.emit(
                TOOL_PROGRESS_EVENT,
                KnowledgeEventDownloadProgress {
                    tool: tool.as_str().to_string(),
                    bytes_downloaded: progress.bytes_downloaded as i64,
                    bytes_total: progress.bytes_total as i64,
                    status: progress.status.to_string(),
                    error_message: progress.error_message.clone(),
                },
            );
        };
        match knowledge::download_tool(tool, emit).await {
            Ok(()) => {
                let _ = handle.emit(
                    TOOL_PROGRESS_EVENT,
                    KnowledgeEventDownloadProgress {
                        tool: tool.as_str().to_string(),
                        bytes_downloaded: 0,
                        bytes_total: 0,
                        status: "done".to_string(),
                        error_message: None,
                    },
                );
            }
            Err(error) => {
                let _ = handle.emit(
                    TOOL_PROGRESS_EVENT,
                    KnowledgeEventDownloadProgress {
                        tool: tool.as_str().to_string(),
                        bytes_downloaded: 0,
                        bytes_total: 0,
                        status: "failed".to_string(),
                        error_message: Some(error.to_string()),
                    },
                );
            }
        }
    });

    Ok(KnowledgeIpcToolDownloadResponse {
        ok: true,
        error_message: None,
    })
}
