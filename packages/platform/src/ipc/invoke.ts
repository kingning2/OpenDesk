/**
 * 带耗时日志的 Tauri IPC 调用封装。
 *
 * 负责：
 * - 统一 React → Rust `invoke` 入口
 * - 记录 command / durationMs / 成败（经 console-bridge 进入日志面板）
 * - 对高频命令降噪
 *
 * @author Xiaoman
 * @created 2026-08-13
 */

import { invoke as tauriInvoke } from "@tauri-apps/api/core";

/** 完全静默：日志面板自身轮询，再打日志会自反馈刷屏。 */
const SILENT_COMMANDS = new Set(["log_recent", "log_clear"]);

/** 高频探测/轮询命令：正常且够快时只打 debug。 */
const QUIET_COMMANDS = new Set([
  "agent_ping",
  "channel_qr_check",
  "license_status",
]);

const QUIET_SLOW_MS = 500;

/**
 * 调用 Tauri command，并输出耗时日志。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @typeParam T - 响应类型
 * @param command - Tauri command 名
 * @param payload - 可选参数对象
 * @returns command 返回值
 */
export async function call<T>(
  command: string,
  payload?: Record<string, unknown>,
): Promise<T> {
  const started = performance.now();
  try {
    const result = await tauriInvoke<T>(command, payload);
    const durationMs = Math.max(0, Math.round(performance.now() - started));
    if (!SILENT_COMMANDS.has(command)) {
      logIpcCompleted(command, durationMs, true);
    }
    return result;
  } catch (error) {
    const durationMs = Math.max(0, Math.round(performance.now() - started));
    // 静默命令失败仍要可见，否则面板拉取异常无从排查。
    logIpcCompleted(command, durationMs, false, error);
    throw error;
  }
}

/**
 * 输出 IPC 完成日志（成功 INFO/DEBUG，失败 WARN）。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param command - command 名
 * @param durationMs - 耗时毫秒
 * @param ok - 是否成功
 * @param error - 失败时的错误
 */
function logIpcCompleted(
  command: string,
  durationMs: number,
  ok: boolean,
  error?: unknown,
): void {
  const message = ok
    ? `IPC 调用完成 command=${command} durationMs=${durationMs}`
    : `IPC 调用失败 command=${command} durationMs=${durationMs} error=${stringifyError(error)}`;

  if (!ok) {
    console.warn(message);
    return;
  }

  const quiet = QUIET_COMMANDS.has(command) && durationMs < QUIET_SLOW_MS;
  if (quiet) {
    console.debug(message);
  } else {
    console.info(message);
  }
}

/**
 * 将未知错误转成短字符串。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param error - 未知错误
 * @returns 可读短文本
 */
function stringifyError(error: unknown): string {
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
