/**
 * 文件选择封装：桌面走原生对话框，浏览器 / web 端走 HTML file input + 上传。
 *
 * 知识库导入统一入口 `pickDocumentFile()`：桌面返回本地磁盘绝对路径；
 * web 端返回文件经 `POST /api/upload` 上传后的 server 临时路径。两种环境下
 * 调用方拿到的都是可直接交给 `knowledge_doc_import` 的路径。
 *
 * @author coisini
 */

import { isTauriRuntime } from "./ipc/invoke";

/** 知识库支持的上传文档扩展名（与后端解析器对齐）。 */
const DOCUMENT_EXTENSIONS = ["pdf", "docx", "txt", "md", "markdown", "html", "htm"];

/**
 * 弹出文件选择并返回可交给 `knowledge_doc_import` 的路径；取消返回 `null`。
 *
 * - 桌面：原生对话框 → 本地绝对路径
 * - web：`<input type="file">` → 上传到 `/api/upload` → server 临时路径
 */
export async function pickDocumentFile(): Promise<string | null> {
  if (isTauriRuntime()) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Documents", extensions: DOCUMENT_EXTENSIONS }],
    });
    return typeof selected === "string" ? selected : null;
  }

  const file = await pickFileFromBrowser();
  if (!file) {
    return null;
  }
  return uploadDocumentFile(file.name, file.blob);
}

/** web 端把文件字节上传到 `/api/upload`，返回 server 临时路径。 */
export async function uploadDocumentFile(name: string, blob: Blob): Promise<string> {
  const base = (typeof window !== "undefined" && (window as { __OPENDESK_SERVER__?: string }).__OPENDESK_SERVER__) || "";
  const form = new FormData();
  form.append("file", blob, name);
  const response = await fetch(`${base}/api/upload`, {
    method: "POST",
    body: form,
  });
  const body = (await response.json()) as { ok: boolean; file_path?: string; error?: string };
  if (!response.ok || body.ok !== true || !body.file_path) {
    throw new Error(body.error ?? `upload failed: HTTP ${response.status}`);
  }
  return body.file_path;
}

/** 浏览器文件选择（web 端知识库导入用）；取消返回 `null`。 */
export function pickFileFromBrowser(): Promise<{ name: string; blob: Blob } | null> {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = DOCUMENT_EXTENSIONS.map((ext) => `.${ext}`).join(",");
    input.onchange = () => {
      const file = input.files?.[0];
      if (!file) {
        resolve(null);
        return;
      }
      resolve({ name: file.name, blob: file });
    };
    input.oncancel = () => resolve(null);
    input.click();
  });
}
