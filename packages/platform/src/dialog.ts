/**
 * 原生文件对话框封装（唯一允许使用 `@tauri-apps/plugin-dialog` 的入口）。
 *
 * 知识库导入由这里弹出系统文件选择窗口，返回磁盘绝对路径；取消返回 `null`。
 * Feature 层禁止直接 `import` plugin-dialog，一律经此封装。
 *
 * @author coisini
 */

import { open } from "@tauri-apps/plugin-dialog";

/** 知识库支持的上传文档扩展名（与后端解析器对齐）。 */
const DOCUMENT_EXTENSIONS = ["pdf", "docx", "txt", "md", "markdown", "html", "htm"];

/**
 * 弹出文件选择窗口，返回选中文档的绝对路径；用户取消时返回 `null`。
 */
export async function pickDocumentFile(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Documents", extensions: DOCUMENT_EXTENSIONS }],
  });
  if (typeof selected === "string") {
    return selected;
  }
  return null;
}
