import { invokeIpc } from "./invoke";
import type {
  KnowledgeDtoDocument,
  KnowledgeDtoToolStatus,
  KnowledgeEventDownloadProgress,
  KnowledgeEventImportUpdated,
  KnowledgeIpcDocumentDeleteRequest,
  KnowledgeIpcDocumentDeleteResponse,
  KnowledgeIpcDocumentImportRequest,
  KnowledgeIpcDocumentImportResponse,
  KnowledgeIpcDocumentListResponse,
  KnowledgeIpcToolDownloadRequest,
  KnowledgeIpcToolDownloadResponse,
  KnowledgeIpcToolStatusResponse,
} from "@desk/contracts";
import { listenEvent } from "../events";

/** 一个知识库文档。 */
export type KnowledgeDocument = KnowledgeDtoDocument;
/** 一个解析工具的安装状态。 */
export type KnowledgeToolStatus = KnowledgeDtoToolStatus;
/** 解析工具下载进度。 */
export type KnowledgeDownloadProgress = KnowledgeEventDownloadProgress;
/** 知识库导入状态变化。 */
export type KnowledgeImportUpdated = KnowledgeEventImportUpdated;

/** 解析工具下载进度事件 topic（与 Rust 对齐）。 */
export const KnowledgeToolProgressEvent = "knowledge:tool/progress" as const;
/** 知识库导入状态变化事件 topic（与 Rust 对齐）。 */
export const KnowledgeImportUpdatedEvent = "knowledge:import/updated" as const;

/**
 * 入队一个知识库导入 job：把本地文件路径交给 worker 解析并向量化。
 */
export async function knowledgeDocImport(
  input: KnowledgeIpcDocumentImportRequest,
): Promise<KnowledgeIpcDocumentImportResponse> {
  return invokeIpc<KnowledgeIpcDocumentImportResponse>("knowledge_doc_import", {
    request: input,
  });
}

/**
 * 列出知识库所有文档（最新更新在前）。
 */
export async function knowledgeDocList(): Promise<KnowledgeDocument[]> {
  const response = await invokeIpc<KnowledgeIpcDocumentListResponse>("knowledge_doc_list");
  try {
    const parsed = JSON.parse(response.documents_json ?? "[]") as KnowledgeDocument[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

/**
 * 删除一个知识库文档（级联删除分块与向量）。
 */
export async function knowledgeDocDelete(
  input: KnowledgeIpcDocumentDeleteRequest,
): Promise<KnowledgeIpcDocumentDeleteResponse> {
  return invokeIpc<KnowledgeIpcDocumentDeleteResponse>("knowledge_doc_delete", { request: input });
}

/**
 * 查询三个解析工具（Pandoc / Tesseract / PDFium）的安装状态。
 */
export async function knowledgeToolStatus(): Promise<KnowledgeToolStatus[]> {
  const response = await invokeIpc<KnowledgeIpcToolStatusResponse>("knowledge_tool_status");
  try {
    const parsed = JSON.parse(response.tools_json ?? "[]") as KnowledgeToolStatus[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

/**
 * 下载并安装一个解析工具；进度经 `knowledge:tool/progress` 事件推送。
 */
export async function knowledgeToolDownload(
  input: KnowledgeIpcToolDownloadRequest,
): Promise<KnowledgeIpcToolDownloadResponse> {
  return invokeIpc<KnowledgeIpcToolDownloadResponse>("knowledge_tool_download", { request: input });
}

/**
 * 订阅解析工具下载进度事件。
 *
 * @returns Promise 解析为取消订阅函数
 */
export async function listenKnowledgeToolProgress(
  handler: (payload: KnowledgeDownloadProgress) => void,
): Promise<() => void> {
  return listenEvent<KnowledgeDownloadProgress>(KnowledgeToolProgressEvent, handler);
}

/**
 * 订阅知识库导入状态变化事件（worker 导入完成后由主进程推送）。
 *
 * @returns Promise 解析为取消订阅函数
 */
export async function listenKnowledgeImportUpdated(
  handler: (payload: KnowledgeImportUpdated) => void,
): Promise<() => void> {
  return listenEvent<KnowledgeImportUpdated>(KnowledgeImportUpdatedEvent, handler);
}
