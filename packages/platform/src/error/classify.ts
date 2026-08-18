/**
 * 错误分类 — 将未知错误按来源与消息特征归为 network / ipc / code。
 *
 * 判定规则：
 * - Rule 0：已带 `kind` 的 DeskError / IpcError 原样返回（幂等）。
 * - Rule 1：消息命中网络标记（transport / sidecar / unavailable / 超时等）→ `network`。
 * - Rule 2：来自 IPC 上下文（带 command）且未命中 → `ipc`。
 * - Rule 3：无 command 的非 IPC 输入 → `code`（防御兜底；代码错误通常由
 *   全局处理器直接构造，不经过此处）。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import type { DeskError, ErrorKind } from "./types";

/** 网络错误消息标记（大小写不敏感子串）。 */
const NETWORK_MARKERS: readonly string[] = [
  // SidecarClientError::Transport（client.rs）
  "transport",
  // reqwest 传输失败
  "connection refused",
  "tcp connect",
  "connect error",
  "error sending request",
  "connection reset",
  "unexpected eof",
  "broken pipe",
  // OS 网络错误码
  "econnrefused",
  "econnreset",
  "enotfound",
  "ehostunreach",
  "etimedout",
  // DingDaError::Unavailable
  "unavailable",
  // 超时
  "timeout",
  "timed out",
  // SidecarClientError::Sidecar（后端返回非 2xx）
  "unexpected status",
  // SidecarLifecycleError（lifecycle.rs）
  "启动侧车失败",
  "侧车启动超时",
  "超过最大重启次数",
  "侧车目录不存在",
  "PATH 中未找到 Python",
  "health check failed",
];

/**
 * 判断对象是否已是分类后的错误（幂等短路）。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param error - 未知错误
 * @returns 命中返回 true
 */
export function isClassified(error: unknown): error is DeskError {
  if (error === null || typeof error !== "object") {
    return false;
  }
  const kind = (error as DeskError).kind;
  return kind === "network" || kind === "ipc" || kind === "code";
}

/**
 * 将未知错误转成短字符串。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param error - 未知错误
 * @returns 可读短文本
 */
export function stringifyError(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  try {
    return JSON.stringify(error) ?? String(error);
  } catch {
    return String(error);
  }
}

/**
 * 对未知错误做分类，返回统一错误载体。
 *
 * 供 IPC 包装与上报入口使用；代码错误由全局处理器直接构造，不经过此处。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param error - 未知错误
 * @param command - 产生错误的 Tauri command 名（IPC 上下文）
 * @returns 分类后的 DeskError
 */
export function classifyError(error: unknown, command?: string): DeskError {
  if (isClassified(error)) {
    return error;
  }
  const message = stringifyError(error);
  const kind = classifyKind(message, command);
  return { kind, message, command };
}

/**
 * 按消息特征与来源判断错误分类。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param message - 错误消息
 * @param command - Tauri command 名（IPC 上下文）
 * @returns 分类
 */
export function classifyKind(message: string, command?: string): ErrorKind {
  const lower = message.toLowerCase();
  if (NETWORK_MARKERS.some((marker) => lower.includes(marker.toLowerCase()))) {
    return "network";
  }
  return command ? "ipc" : "code";
}
