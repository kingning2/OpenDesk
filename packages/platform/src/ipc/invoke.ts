/**
 * Tauri IPC：调用封装、错误归一化、日志落盘、前端错误上报。
 *
 * @author Xiaoman
 * @created 2026-08-01
 */

import { invoke } from "@tauri-apps/api/core";

/** 写日志用的内部 command，自身不再记 IPC 日志。 */
export const DIAGNOSTICS_LOG_COMMAND = "diagnostics_log";

/** 前端错误类型。 */
export type FrontendErrorKind = "uncaught" | "unhandledrejection" | "react" | "manual";

/** 前端错误上报字段。 */
export interface FrontendErrorReport {
  kind: FrontendErrorKind;
  message: string;
  route?: string;
  source?: string;
  line?: number;
  column?: number;
  component?: string;
  stack?: string;
  detail?: string;
}

/** IPC 失败时的桌面端回调（例如 toast）。 */
export type InvokeErrorReporter = (
  command: string,
  message: string,
  error: InvokeError,
) => void;

let invokeErrorReporter: InvokeErrorReporter | null = null;
const toastKeys = new Set<string>();
const seenFrontendErrors = new Set<string>();

/** Tauri IPC 失败时抛出的错误。 */
export class InvokeError extends Error {
  readonly raw: unknown;
  readonly command: string;

  constructor(command: string, message: string, raw: unknown) {
    super(message);
    this.name = "InvokeError";
    this.command = command;
    this.raw = raw;
  }
}

/** 注册 IPC 失败回调。 */
export function setInvokeErrorReporter(reporter: InvokeErrorReporter | null): void {
  invokeErrorReporter = reporter;
}

/** 把 Tauri 拒绝值转成可读错误文案。 */
export function formatInvokeError(error: unknown): string {
  if (typeof error === "string") return error.trim() || "Backend request failed";
  if (error instanceof Error) return error.message.trim() || "Backend request failed";
  if (error && typeof error === "object") {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string" && message.trim()) return message.trim();
  }
  return String(error).trim() || "Backend request failed";
}

/** JSON 序列化，失败时退回字符串。 */
function toJson(value: unknown): string {
  try {
    return JSON.stringify(value ?? {});
  } catch {
    return String(value ?? "");
  }
}

/** 当前页面路由，用于日志关联。 */
function currentRoute(): string {
  if (typeof window === "undefined") return "";
  return `${window.location.pathname}${window.location.search}${window.location.hash}`;
}

/** 从 stack 里解析第一个业务代码位置。 */
function parseStackLocation(stack?: string): Pick<FrontendErrorReport, "source" | "line" | "column"> {
  if (!stack) return {};
  for (const line of stack.split("\n")) {
    const matched = line.trim().match(/(?:at\s+(?:.*?\s+)?\(?)(.+?):(\d+):(\d+)\)?$/);
    if (matched?.[1] && !matched[1].includes("node_modules")) {
      return { source: matched[1], line: Number(matched[2]), column: Number(matched[3]) };
    }
  }
  return {};
}

/** 写一行日志到 Rust 本地文件，失败时静默。 */
export function writeDiagnosticsLog(
  level: string,
  event: string,
  input: unknown,
  output: unknown,
): void {
  void invoke(DIAGNOSTICS_LOG_COMMAND, {
    request: { level, event, input: toJson(input), output: toJson(output) },
  }).catch(() => {});
}

/** 上报前端错误（自动去重）。 */
export function reportFrontendError(report: FrontendErrorReport): void {
  const payload = { ...parseStackLocation(report.stack), ...report, route: report.route ?? currentRoute() };
  const key = toJson(payload);
  if (seenFrontendErrors.has(key)) return;
  seenFrontendErrors.add(key);

  writeDiagnosticsLog("ERROR", "frontend.error", payload, {
    message: payload.message,
    stack: payload.stack,
    detail: payload.detail,
  });
}

/** 上报 unknown 类型的错误值。 */
export function reportFrontendErrorValue(
  kind: FrontendErrorKind,
  error: unknown,
  context: Omit<FrontendErrorReport, "kind" | "message" | "stack"> = {},
): void {
  const message =
    error instanceof Error ? error.message || error.name : typeof error === "string" ? error : toJson(error);
  const stack = error instanceof Error ? error.stack : undefined;
  reportFrontendError({ kind, message, stack, ...parseStackLocation(stack), ...context });
}

/** 记录 IPC 调用（跳过 diagnostics_log 自身）。 */
function logIpc(level: "INFO" | "ERROR", command: string, input: string, output: string): void {
  if (command !== DIAGNOSTICS_LOG_COMMAND) {
    writeDiagnosticsLog(level, command, input, output);
  }
}

/** 调用 Tauri command，并写入 `北京时间【LEVEL】【事件】【入参】【出参】` 日志。 */
export async function invokeIpc<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const input = toJson(args ?? {});
  try {
    const result = await invoke<T>(command, args);
    logIpc("INFO", command, input, toJson(result));
    return result;
  } catch (error) {
    const invokeError =
      error instanceof InvokeError ? error : new InvokeError(command, formatInvokeError(error), error);
    logIpc("ERROR", command, input, invokeError.message);
    invokeErrorReporter?.(invokeError.command, invokeError.message, invokeError);
    throw invokeError;
  }
}

/** 安装 IPC 失败 toast（相同 command+message 只弹一次）。 */
export function installIpcErrorReporter(onError: (command: string, message: string) => void): void {
  setInvokeErrorReporter((command, message) => {
    const key = `${command}\0${message}`;
    if (toastKeys.has(key)) return;
    toastKeys.add(key);
    onError(command, message);
  });
}
