/**
 * 错误信息提取工具（抽取自原 xianyu 前端 utils/apiError.ts 语义）。
 */

/**
 * 从任意错误中提取可读的错误文案。
 *
 * 兼容：
 * - `Error` 实例（取 message）
 * - 字符串
 * - 统一响应包装 `{ message }` / `{ error }` / `{ data: { message } }`
 * - Tauri IPC 的 `{ error }` 结构
 *
 * @param error 未知错误
 * @param fallback 兜底文案
 * @returns 可读错误文本
 */
export function getErrorMessage(error: unknown, fallback = "操作失败"): string {
  if (typeof error === "string" && error.trim()) {
    return error;
  }
  if (error instanceof Error && error.message) {
    return error.message;
  }
  if (error && typeof error === "object") {
    const record = error as Record<string, unknown>;
    const candidates = ["message", "error", "detail"];
    for (const key of candidates) {
      const value = record[key];
      if (typeof value === "string" && value.trim()) {
        return value;
      }
      if (value && typeof value === "object") {
        const nested = getErrorMessage(value, "");
        if (nested) {
          return nested;
        }
      }
    }
  }
  return fallback;
}

/**
 * 兼容别名：原前端导出名 `getApiErrorMessage`。
 */
export const getApiErrorMessage = getErrorMessage;
